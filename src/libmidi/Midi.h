// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __MIDI_H
#define __MIDI_H

#include <iostream>
#include <vector>

#include "Note.h"
#include "MidiTrack.h"
#include "MidiTypes.h"

class MidiError;
class MidiEvent;

typedef std::vector<MidiTrack> MidiTrackList;

typedef std::vector<MidiEvent> MidiEventList;
typedef std::vector<std::pair<size_t, MidiEvent> > MidiEventListWithTrackId;

// NOTE: This library's MIDI loading and handling is destructive.  Perfect
//       1:1 serialization routines will not be possible without quite a
//       bit of additional work.
class Midi {

public:
  static Midi ReadFromFile(const std::string &filename);
  static Midi ReadFromStream(std::istream &stream);

  const std::vector<MidiTrack> &Tracks() const {
    return m_tracks;
  }

  const TranslatedNoteSet &Notes() const {
    return m_translated_notes;
  }

  MidiEventListWithTrackId Update(microseconds_t delta_microseconds);
  void GoTo(microseconds_t microsecond_song_position);

  void Reset(microseconds_t lead_in_microseconds, 
	     microseconds_t lead_out_microseconds);

  microseconds_t GetSongPositionInMicroseconds() const {
    return m_microsecond_song_position;
  }

  microseconds_t GetSongLengthInMicroseconds() const;

  microseconds_t GetDeadAirStartOffsetMicroseconds() const {
    return m_microsecond_dead_start_air;
  }

  // This doesn't include lead-in (so it's perfect for a progress bar).
  // (It is also clamped to [0.0, 1.0], so lead-in and lead-out won't give any
  // unexpected results.)
  double GetSongPercentageComplete() const;

  // This will report when the lead-out period is complete.
  bool IsSongOver() const;

  unsigned int AggregateEventsRemain() const;
  unsigned int AggregateEventCount() const;

  unsigned int AggregateNotesRemain() const;
  unsigned int AggregateNoteCount() const;

  const MidiEventMicrosecondList & GetBarLines() const {
   return m_bar_line_usecs;
  }

  microseconds_t GetNextBarInMicroseconds(const microseconds_t point) const;

private:
  const static unsigned long DefaultBPM = 120;
  const static microseconds_t OneMinuteInMicroseconds = 60000000;
  const static microseconds_t DefaultUSTempo = OneMinuteInMicroseconds / DefaultBPM;

  static microseconds_t ConvertPulsesToMicroseconds(unsigned long pulses,
                                                    microseconds_t tempo,
                                                    unsigned short pulses_per_quarter_note);

  Midi():
    m_initialized(false), m_microsecond_dead_start_air(0) {

    Reset(0, 0);
  }

  // This is O(n) where n is the number of tempo changes (across all tracks) in
  // the song up to the specified time.  Tempo changes are usually a small number.
  // (Almost always 0 or 1, going up to maybe 30-100 in rare cases.)
  microseconds_t GetEventPulseInMicroseconds(unsigned long event_pulses,
                                             unsigned short pulses_per_quarter_note) const;

  unsigned long FindFirstNotePulse();

  void BuildTempoTrack();
  void TranslateNotes(const NoteSet &notes, unsigned short pulses_per_quarter_note);

  bool m_initialized;

  TranslatedNoteSet m_translated_notes;

  // Position can be negative (for lead-in).
  microseconds_t m_microsecond_song_position;
  microseconds_t m_microsecond_base_song_length;

  microseconds_t m_microsecond_lead_in;
  microseconds_t m_microsecond_lead_out;
  microseconds_t m_microsecond_dead_start_air;

  bool m_first_update_after_reset;
  double m_playback_speed;
  MidiTrackList m_tracks;
  MidiEventMicrosecondList m_bar_line_usecs;
};

#endif
