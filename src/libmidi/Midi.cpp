// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "Midi.h"
#include "MidiEvent.h"
#include "MidiTrack.h"
#include "MidiUtil.h"

#include <fstream>
#include <map>
#include <stdint.h>

using namespace std;

Midi Midi::ReadFromFile(const string &filename) {

  fstream file(filename.c_str(), ios::in | ios::binary);

  if (!file.good())
    throw MidiError(MidiError_BadFilename);

  Midi m;

  try {
    m = ReadFromStream(file);
  }

  catch (const MidiError &e) {
    // Close our file resource before handing the error up
    file.close();
    throw e;
  }

  return m;
}

Midi Midi::ReadFromStream(istream &stream) {
  Midi m;

  // header_id is always "MThd" by definition
  const static string MidiFileHeader = "MThd";
  const static string RiffFileHeader = "RIFF";

  // I could use (MidiFileHeader.length() + 1), but then this has to be
  // dynamically allocated.  More hassle than it's worth.  MIDI is well
  // defined and will always have a 4-byte header.  We use 5 so we get
  // free null termination.
  char           header_id[5] = { 0, 0, 0, 0, 0 };
  uint32_t header_length;
  unsigned short format;
  unsigned short track_count;
  unsigned short time_division;

  stream.read(header_id, static_cast<streamsize>(MidiFileHeader.length()));
  string header(header_id);
  if (header != MidiFileHeader) {
    if (header != RiffFileHeader)
      throw MidiError(MidiError_UnknownHeaderType);

    else {
      // We know how to support RIFF files
      unsigned long throw_away;
      stream.read(reinterpret_cast<char*>(&throw_away), sizeof(unsigned long)); // RIFF length
      stream.read(reinterpret_cast<char*>(&throw_away), sizeof(unsigned long)); // "RMID"
      stream.read(reinterpret_cast<char*>(&throw_away), sizeof(unsigned long)); // "data"
      stream.read(reinterpret_cast<char*>(&throw_away), sizeof(unsigned long)); // data size

      // Call this recursively, without the RIFF header this time
      return ReadFromStream(stream);
    }
  }

  stream.read(reinterpret_cast<char*>(&header_length), sizeof(uint32_t));
  stream.read(reinterpret_cast<char*>(&format),        sizeof(unsigned short));
  stream.read(reinterpret_cast<char*>(&track_count),   sizeof(unsigned short));
  stream.read(reinterpret_cast<char*>(&time_division), sizeof(unsigned short));

  if (stream.fail())
    throw MidiError(MidiError_NoHeader);

  // Chunk Size is always 6 by definition
  const static unsigned int MidiFileHeaderChunkLength = 6;

  header_length = BigToSystem32(header_length);
  if (header_length != MidiFileHeaderChunkLength)
    throw MidiError(MidiError_BadHeaderSize);

  enum MidiFormat { MidiFormat0 = 0, MidiFormat1, MidiFormat2 };

  format = BigToSystem16(format);
  if (format == MidiFormat2) {
    // MIDI 0: All information in 1 track
    // MIDI 1: Multiple tracks intended to be played simultaneously
    // MIDI 2: Multiple tracks intended to be played separately
    //
    // We do not support MIDI 2 at this time
    throw MidiError(MidiError_Type2MidiNotSupported);
  }

  track_count = BigToSystem16(track_count);
  if (format == 0 && track_count != 1)
    // MIDI 0 has only 1 track by definition
    throw MidiError(MidiError_BadType0Midi);

  // Time division can be encoded two ways based on a bit-flag:
  // - pulses per quarter note (15-bits)
  // - SMTPE frames per second (7-bits for SMPTE frame count and 8-bits for clock ticks per frame)
  time_division = BigToSystem16(time_division);
  bool in_smpte = ((time_division & 0x8000) != 0);

  if (in_smpte)
    throw MidiError(MidiError_SMTPETimingNotImplemented);

  // We ignore the possibility of SMPTE timing, so we can
  // use the time division value directly as PPQN.
  unsigned short pulses_per_quarter_note = time_division;

   // Read in our tracks
  for (int i = 0; i < track_count; ++i) {
    m.m_tracks.push_back(MidiTrack::ReadFromStream(stream));
  }

  m.BuildTempoTrack();

  // Tell our tracks their IDs
  for (int i = 0; i < track_count; ++i) {
    m.m_tracks[i].SetTrackId(i);
  }

   // Translate each track's list of notes and list
   // of events into microseconds.
   for (MidiTrackList::iterator i = m.m_tracks.begin(); i != m.m_tracks.end(); ++i) {
     i->Reset();
     m.TranslateNotes(i->Notes(), pulses_per_quarter_note);

     MidiEventMicrosecondList event_usecs;
     for (MidiEventPulsesList::const_iterator j = i->EventPulses().begin(); j != i->EventPulses().end(); ++j) {
       event_usecs.push_back(m.GetEventPulseInMicroseconds(*j, pulses_per_quarter_note));
     }

     i->SetEventUsecs(event_usecs);
   }

   m.m_initialized = true;

   // Just grab the end of the last note to find out how long the song is
   m.m_microsecond_base_song_length = m.m_translated_notes.rbegin()->end;

   // Eat everything up until *just* before the first note event
   m.m_microsecond_dead_start_air = m.GetEventPulseInMicroseconds(m.FindFirstNotePulse(), pulses_per_quarter_note) - 1;

   // Calculate positions for bar_lines
   MidiEventMicrosecondList bar_line_usecs;
   const microseconds_t len = m.GetSongLengthInMicroseconds();
   microseconds_t bar_usec = 0;
   int bar_no = 0;
   while (bar_usec <= len)
   {
       bar_usec = m.GetEventPulseInMicroseconds(bar_no*pulses_per_quarter_note*4, pulses_per_quarter_note);
       bar_line_usecs.push_back(bar_usec);
       bar_no++;
   }
   m.m_bar_line_usecs = bar_line_usecs;

   return m;
}

// NOTE: This is required for much of the other functionality provided
// by this class, however, this causes a destructive change in the way
// the MIDI is represented internally which means we can never save the
// file back out to disk exactly as we loaded it.
//
// This adds an extra track dedicated to tempo change events.  Tempo events
// are extracted from every other track and placed in the new one.
//
// This allows quick(er) calculation of wall-clock event times
void Midi::BuildTempoTrack() {
  // This map will help us get rid of duplicate events if
  // the tempo is specified in every track (as is common).
  //
  // It also does sorting for us so we can just copy the
  // events right over to the new track.
  map<unsigned long, MidiEvent> tempo_events;

  // Run through each track looking for tempo events
  for (MidiTrackList::iterator t = m_tracks.begin(); t != m_tracks.end(); ++t) {
    for (size_t i = 0; i < t->Events().size(); ++i) {
      MidiEvent ev = t->Events()[i];
      unsigned long ev_pulses = t->EventPulses()[i];

      if (ev.Type() == MidiEventType_Meta &&
          ev.MetaType() == MidiMetaEvent_TempoChange) {

        // Pull tempo event out of both lists
        //
        // Vector is kind of a hassle this way -- we have to
        // walk an iterator to that point in the list because
        // erase MUST take an iterator... but erasing from a
        // list invalidates iterators.  bleah.
        MidiEventList::iterator event_to_erase = t->Events().begin();
        MidiEventPulsesList::iterator event_pulse_to_erase = t->EventPulses().begin();
        for (size_t j = 0; j < i; ++j) {
          ++event_to_erase;
          ++event_pulse_to_erase;
        }

        t->Events().erase(event_to_erase);
        t->EventPulses().erase(event_pulse_to_erase);

        // Adjust next event's delta time
        if (t->Events().size() > i) {
          // (We just erased the element at i, so
          // now i is pointing to the next element)
          unsigned long next_dt = t->Events()[i].GetDeltaPulses();

          t->Events()[i].SetDeltaPulses(ev.GetDeltaPulses() + next_dt);
        }

        // We have to roll i back for the next loop around
        --i;

        // Insert our newly stolen event into the auto-sorting map
        tempo_events[ev_pulses] = ev;
      }
    }
  }

  // Create a new track (always the last track in the track list)
  m_tracks.push_back(MidiTrack::CreateBlankTrack());

  MidiEventList &tempo_track_events = m_tracks[m_tracks.size()-1].Events();
  MidiEventPulsesList &tempo_track_event_pulses = m_tracks[m_tracks.size()-1].EventPulses();

  // Copy over all our tempo events
  unsigned long previous_absolute_pulses = 0;
  for (map<unsigned long, MidiEvent>::const_iterator i = tempo_events.begin(); 
       i != tempo_events.end(); ++i) {

    unsigned long absolute_pulses = i->first;
    MidiEvent ev = i->second;

    // Reset each of their delta times while we go
    ev.SetDeltaPulses(absolute_pulses - previous_absolute_pulses);
    previous_absolute_pulses = absolute_pulses;

    // Add them to the track
    tempo_track_event_pulses.push_back(absolute_pulses);
    tempo_track_events.push_back(ev);
  }
}

unsigned long Midi::FindFirstNotePulse() {
  unsigned long first_note_pulse = 0;

  // Find the very last value it could ever possibly be, to start with
  for (MidiTrackList::const_iterator t = m_tracks.begin(); t != m_tracks.end(); ++t) {
    if (t->EventPulses().size() == 0)
      continue;

    unsigned long pulses = t->EventPulses().back();

    if (pulses > first_note_pulse)
      first_note_pulse = pulses;
  }

  // Now run through each event in each track looking for the very
  // first note_on event
  for (MidiTrackList::const_iterator t = m_tracks.begin(); t != m_tracks.end(); ++t) {
    for (size_t ev_id = 0; ev_id < t->Events().size(); ++ev_id) {
      if (t->Events()[ev_id].Type() == MidiEventType_NoteOn) {
        unsigned long note_pulse = t->EventPulses()[ev_id];

        if (note_pulse < first_note_pulse)
          first_note_pulse = note_pulse;

        // We found the first note event in this
        // track.  No need to keep searching.
        break;
      }
    }
  }

  return first_note_pulse;
}

microseconds_t Midi::ConvertPulsesToMicroseconds(unsigned long pulses,
                                                 microseconds_t tempo,
                                                 unsigned short pulses_per_quarter_note) {
  // Here's what we have to work with:
  //   pulses is given
  //   tempo is given (units of microseconds/quarter_note)
  //   (pulses/quarter_note) is given as a constant in this object file
  const double quarter_notes = static_cast<double>(pulses) / static_cast<double>(pulses_per_quarter_note);
  const double microseconds = quarter_notes * static_cast<double>(tempo);

  return static_cast<microseconds_t>(microseconds);
}

microseconds_t Midi::GetEventPulseInMicroseconds(unsigned long event_pulses,
                                                 unsigned short pulses_per_quarter_note) const {
  if (m_tracks.size() == 0)
    return 0;

  const MidiTrack &tempo_track = m_tracks.back();

  microseconds_t running_result = 0;

  bool hit = false;
  unsigned long last_tempo_event_pulses = 0;
  microseconds_t running_tempo = DefaultUSTempo;

  for (size_t i = 0; i < tempo_track.Events().size(); ++i) {
    unsigned long tempo_event_pulses = tempo_track.EventPulses()[i];

    // If the time we're asking to convert is still beyond
    // this tempo event, just add the last time slice (at
    // the previous tempo) to the running wall-clock time.
    unsigned long delta_pulses = 0;
    if (event_pulses > tempo_event_pulses)
      delta_pulses = tempo_event_pulses - last_tempo_event_pulses;

    else {
      hit = true;
      delta_pulses = event_pulses - last_tempo_event_pulses;
    }

    running_result += ConvertPulsesToMicroseconds(delta_pulses, running_tempo, pulses_per_quarter_note);

    // If the time we're calculating is before the tempo event we're
    // looking at, we're done.
    if (hit)
      break;

    running_tempo = tempo_track.Events()[i].GetTempoInUsPerQn();
    last_tempo_event_pulses = tempo_event_pulses;
  }

  // The requested time may be after the very last tempo event
  if (!hit) {
    unsigned long remaining_pulses = event_pulses - last_tempo_event_pulses;
    running_result += ConvertPulsesToMicroseconds(remaining_pulses, running_tempo, pulses_per_quarter_note);
  }

  return running_result;
}

void Midi::Reset(microseconds_t lead_in_microseconds, microseconds_t lead_out_microseconds) {
  m_microsecond_lead_in = lead_in_microseconds;
  m_microsecond_lead_out = lead_out_microseconds;
  m_microsecond_song_position = m_microsecond_dead_start_air - lead_in_microseconds;
  m_first_update_after_reset = true;

  for (MidiTrackList::iterator i = m_tracks.begin(); i != m_tracks.end(); ++i) {
    i->Reset();
  }
}

void Midi::TranslateNotes(const NoteSet &notes, unsigned short pulses_per_quarter_note) {
  for (NoteSet::const_iterator i = notes.begin(); i != notes.end(); ++i) {
    TranslatedNote trans;

    trans.note_id = i->note_id;
    trans.track_id = i->track_id;
    trans.channel = i->channel;
    trans.velocity = i->velocity;
    trans.start = GetEventPulseInMicroseconds(i->start, pulses_per_quarter_note);
    trans.end = GetEventPulseInMicroseconds(i->end, pulses_per_quarter_note);

    m_translated_notes.insert(trans);
  }
}

MidiEventListWithTrackId Midi::Update(microseconds_t delta_microseconds) {
  MidiEventListWithTrackId aggregated_events;
  if (!m_initialized)
    return aggregated_events;

  // Move everything forward (fallen keys, the screen keyboard)
  // These variable is used on redraw later
  m_microsecond_song_position += delta_microseconds;
  if (m_first_update_after_reset) {
    delta_microseconds += m_microsecond_song_position;
    m_first_update_after_reset = false;
  }

  if (delta_microseconds == 0)
    return aggregated_events;

  if (m_microsecond_song_position < 0)
    return aggregated_events;

  if (delta_microseconds > m_microsecond_song_position)
    delta_microseconds = m_microsecond_song_position;

  const size_t track_count = m_tracks.size();
  // These code is not related to fallen keys
  for (size_t i = 0; i < track_count; ++i) {
    MidiEventList track_events = m_tracks[i].Update(delta_microseconds);

    const size_t event_count = track_events.size();
    // Collect events to be passed to a screen keyboard
    for (size_t j = 0; j < event_count; ++j) {
      aggregated_events.insert(aggregated_events.end(), make_pair(i, track_events[j]));
    }
  }

  // Pass to a keyboard
  return aggregated_events;
}

void Midi::GoTo(microseconds_t microsecond_song_position) {
  if (!m_initialized)
    return;

  // Do not let go back too far (causes bugs)
  // There is some black magic for negative values of
  // microsecond_song_position, just skip it
//if (microsecond_song_position <= 0)
//{
//    Reset(m_microsecond_lead_in, m_microsecond_lead_out);
//    return;
//}

  m_microsecond_song_position = microsecond_song_position;

  const size_t track_count = m_tracks.size();
  for (size_t i = 0; i < track_count; ++i) {
    m_tracks[i].GoTo(microsecond_song_position);
  }
}

microseconds_t Midi::GetSongLengthInMicroseconds() const {
  if (!m_initialized)
    return 0;

  return m_microsecond_base_song_length - m_microsecond_dead_start_air;
}


// Gets next bar after point of time
microseconds_t Midi::GetNextBarInMicroseconds(const microseconds_t point) const {
   MidiEventMicrosecondList::const_iterator j = m_bar_line_usecs.begin();
   microseconds_t first_bar_usec = *j;
   for (; j != m_bar_line_usecs.end(); ++j) {
     microseconds_t bar_usec = *j;
     // Add offset
     bar_usec -= first_bar_usec + 1;
     if (bar_usec > point)
       return bar_usec;
   }
   return 0; // not found
}

unsigned int Midi::AggregateEventsRemain() const {
  if (!m_initialized)
    return 0;

  unsigned int aggregate = 0;
  for (MidiTrackList::const_iterator i = m_tracks.begin(); i != m_tracks.end(); ++i)
    aggregate += i->AggregateEventsRemain();

  return aggregate;
}

unsigned int Midi::AggregateNotesRemain() const {
  if (!m_initialized)
    return 0;

  unsigned int aggregate = 0;
  for (MidiTrackList::const_iterator i = m_tracks.begin(); i != m_tracks.end(); ++i)
    aggregate += i->AggregateNotesRemain();

  return aggregate;
}

unsigned int Midi::AggregateEventCount() const {
  if (!m_initialized)
    return 0;

  unsigned int aggregate = 0;
  for (MidiTrackList::const_iterator i = m_tracks.begin(); i != m_tracks.end(); ++i)
    aggregate += i->AggregateEventCount();

  return aggregate;
}

unsigned int Midi::AggregateNoteCount() const {
  if (!m_initialized)
    return 0;

  unsigned int aggregate = 0;
  for (MidiTrackList::const_iterator i = m_tracks.begin(); i != m_tracks.end(); ++i)
    aggregate += i->AggregateNoteCount();

  return aggregate;
}

// This function is used for the progress bar
double Midi::GetSongPercentageComplete() const {
  if (!m_initialized)
    return 0.0;

  const double pos = static_cast<double>(m_microsecond_song_position - m_microsecond_dead_start_air);
  const double len = static_cast<double>(GetSongLengthInMicroseconds());

  if (pos < 0)
    return 0.0;

  if (len == 0)
    return 1.0;

  return min( (pos / len), 1.0 );
}

bool Midi::IsSongOver() const {
  if (!m_initialized)
    return true;

  return (m_microsecond_song_position - m_microsecond_dead_start_air) >=
    GetSongLengthInMicroseconds() + m_microsecond_lead_out;
}
