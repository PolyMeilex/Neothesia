// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __MIDI_NOTE_H
#define __MIDI_NOTE_H

#include <set>
#include "MidiTypes.h"

// Range of all 128 MIDI notes possible
typedef unsigned int NoteId;

// Arbitrary value outside the usual range
const static NoteId InvalidNoteId = 2048;

enum NoteState {

   AutoPlayed,
   UserPlayable,
   UserHit,
   UserMissed
};

template <class T>
struct GenericNote  {

  bool operator()(const GenericNote<T> &lhs, const GenericNote<T> &rhs) const {
    if (lhs.start < rhs.start) return true;
    if (lhs.start > rhs.start) return false;

    if (lhs.end < rhs.end) return true;
    if (lhs.end > rhs.end) return false;

    if (lhs.note_id < rhs.note_id) return true;
    if (lhs.note_id > rhs.note_id) return false;

    if (lhs.track_id < rhs.track_id) return true;
    if (lhs.track_id > rhs.track_id) return false;

    return false;
  }

  T start;
  T end;
  NoteId note_id;
  size_t track_id;

  // We have to drag a little extra info around so we can
  // play the user's input correctly
  unsigned char channel;
  int velocity;

  NoteState state;
  // State before retry (last try)
  NoteState retry_state;
};

// Note keeps the internal pulses found in the MIDI file which are
// independent of tempo or playback speed.  TranslatedNote contains
// the exact (translated) microsecond that notes start and stop on
// based on a given playback speed, after dereferencing tempo changes.
typedef GenericNote<unsigned long> Note;
typedef GenericNote<microseconds_t> TranslatedNote;

typedef std::set<Note, Note> NoteSet;
typedef std::set<TranslatedNote, TranslatedNote> TranslatedNoteSet;

#endif
