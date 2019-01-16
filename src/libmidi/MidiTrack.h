// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __MIDI_TRACK_H
#define __MIDI_TRACK_H

#include <vector>
#include <iostream>

#include "Note.h"
#include "MidiEvent.h"
#include "MidiUtil.h"

class MidiEvent;

typedef std::vector<MidiEvent> MidiEventList;
typedef std::vector<unsigned long> MidiEventPulsesList;
typedef std::vector<microseconds_t> MidiEventMicrosecondList;

class MidiTrack {
public:
  static MidiTrack ReadFromStream(std::istream &stream);
  static MidiTrack CreateBlankTrack() {
    return MidiTrack();
  }

  MidiEventList &Events() {
    return m_events;
  }

  MidiEventPulsesList &EventPulses() {
    return m_event_pulses;
  }

  MidiEventMicrosecondList &EventUsecs() {
    return m_event_usecs;
  }

  const MidiEventList &Events() const {
    return m_events;
  }

  const MidiEventPulsesList &EventPulses() const {
    return m_event_pulses;
  }

  const MidiEventMicrosecondList &EventUsecs() const {
    return m_event_usecs;
  }

  void SetEventUsecs(const MidiEventMicrosecondList &event_usecs) {
    m_event_usecs = event_usecs;
  }

  const std::string InstrumentName() const {
    return InstrumentNames[m_instrument_id];
  }

  bool IsPercussion() const {
    return m_instrument_id == InstrumentIdPercussion;
  }

  const NoteSet &Notes() const {
    return m_note_set;
  }

  void SetTrackId(size_t track_id);

  // Reports whether this track contains any Note-On MIDI events
  // (vs. just being an information track with a title or copyright)
  bool hasNotes() const {
    return (m_note_set.size() > 0);
  }

  void Reset();
  MidiEventList Update(microseconds_t delta_microseconds);
  void GoTo(microseconds_t microsecond_song_position);

  unsigned int AggregateEventsRemain() const {
    return static_cast<unsigned int>(m_events.size() - (m_last_event + 1));
  }

  unsigned int AggregateEventCount() const {
    return static_cast<unsigned int>(m_events.size());
  }

  unsigned int AggregateNotesRemain() const {
    return m_notes_remaining;
  }

  unsigned int AggregateNoteCount() const {
    return static_cast<unsigned int>(m_note_set.size());
  }

private:
  MidiTrack() :
    m_instrument_id(0) {

    Reset();
  }

  void BuildNoteSet();
  void DiscoverInstrument();

  MidiEventList m_events;
  MidiEventPulsesList m_event_pulses;
  MidiEventMicrosecondList m_event_usecs;

  NoteSet m_note_set;

  int m_instrument_id;

  microseconds_t m_running_microseconds;
  long m_last_event;

  unsigned int m_notes_remaining;
};

#endif
