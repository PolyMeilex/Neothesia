// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __PLAYING_STATE_H
#define __PLAYING_STATE_H

#include <string>
#include <vector>
#include <set>

#include "libmidi/Midi.h"
#include "SharedState.h"
#include "GameState.h"
#include "KeyboardDisplay.h"
#include "MidiComm.h"

struct ActiveNote {

  bool operator()(const ActiveNote &lhs, const ActiveNote &rhs) {
    if (lhs.note_id < rhs.note_id)
      return true;

    if (lhs.note_id > rhs.note_id)
      return false;

    if (lhs.channel < rhs.channel)
      return true;

    if (lhs.channel > rhs.channel)
      return false;

    return false;
  }

  NoteId note_id;
  unsigned char channel;
  int velocity;
};

typedef std::set<ActiveNote, ActiveNote> ActiveNoteSet;

class PlayingState : public GameState {
public:
  PlayingState(const SharedState &state);
  ~PlayingState();
  bool ResetKeyboardActive();

protected:
  virtual void Init();
  virtual void Update();
  virtual void Draw(Renderer &renderer) const;

private:

  std::set<int> m_pressed_notes;
  std::set<int> m_required_notes;

  void userPressedKey(int note_number, bool active);
  void filePressedKey(int note_number, bool active, size_t track_id);
  bool areAllRequiredKeysPressed();
  bool isKeyPressed(int note_number);
  bool isUserPlayableTrack(size_t track_id);

  int CalcKeyboardHeight() const;
  void SetupNoteState();

  void ResetSong();
  void Play(microseconds_t delta_microseconds);
  void Listen();

  double CalculateScoreMultiplier() const;

  bool m_paused;

  KeyboardDisplay *m_keyboard;
  microseconds_t m_show_duration;
  TranslatedNoteSet m_notes;
  TranslatedNoteSet m_notes_history;

  bool m_any_you_play_tracks;
  size_t m_look_ahead_you_play_note_count;

  ActiveNoteSet m_active_notes;

  bool m_first_update;

  SharedState m_state;
  int m_current_combo;

  double m_title_alpha;
  double m_max_allowed_title_alpha;

  // For octave sliding
  int m_note_offset;

  // For retries
  bool m_should_retry;
  bool m_should_wait_after_retry;
  microseconds_t m_retry_start;

  void eraseUntilTime(microseconds_t time);

  NoteState findNodeState(const TranslatedNote& note, TranslatedNoteSet& notes, NoteState default_note_state);
};

#endif // __PLAYING_STATE_H
