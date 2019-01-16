// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "PlayingState.h"
#include "TrackSelectionState.h"
#include "StatsState.h"
#include "Renderer.h"
#include "Textures.h"
#include "CompatibleSystem.h"

#include <string>
#include <iomanip>

#include "StringUtil.h"
#include "MenuLayout.h"
#include "TextWriter.h"

#include "libmidi/Midi.h"
#include "libmidi/MidiTrack.h"
#include "libmidi/MidiEvent.h"
#include "libmidi/MidiUtil.h"

#include "MidiComm.h"

using namespace std;


void PlayingState::SetupNoteState() {

  TranslatedNoteSet old = m_notes;
  m_notes.clear();

  for (TranslatedNoteSet::const_iterator i = old.begin(); i != old.end(); ++i) {
    TranslatedNote n = *i;

    n.state = AutoPlayed;
    n.retry_state = AutoPlayed;
    if (isUserPlayableTrack(n.track_id))
    {
      n.state = UserPlayable;
      n.retry_state = UserPlayable;
    }

    m_notes.insert(n);
  }
}

void PlayingState::ResetSong() {

  if (m_state.midi_out)
    m_state.midi_out->Reset();

  if (m_state.midi_in)
    m_state.midi_in->Reset();

  // TODO: These should be moved to a configuration file
  // along with ALL other "const static something" variables.
  const static microseconds_t LeadIn = 5500000;
  const static microseconds_t LeadOut = 1000000;

  if (!m_state.midi)
    return;

  m_state.midi->Reset(LeadIn, LeadOut);

  m_notes = m_state.midi->Notes();
  m_notes_history.clear();
  SetupNoteState();

  m_state.stats = SongStatistics();
  m_state.stats.total_note_count = static_cast<int>(m_notes.size());

  m_current_combo = 0;

  m_note_offset = 0;
  m_max_allowed_title_alpha = 1.0;

  m_should_retry = false;
  m_should_wait_after_retry = false;
  m_retry_start = m_state.midi->GetNextBarInMicroseconds(-1000000000);
}

PlayingState::PlayingState(const SharedState &state) :
  m_paused(false),
  m_keyboard(0),
  m_any_you_play_tracks(false),
  m_first_update(true),
  m_should_retry(false),
  m_should_wait_after_retry(false),
  m_retry_start(0),
  m_state(state) {
}

void PlayingState::Init() {

  if (!m_state.midi)
    throw GameStateError("PlayingState: Init was passed a null MIDI!");

  m_look_ahead_you_play_note_count = 0;
  for (size_t i = 0; i < m_state.track_properties.size(); ++i) {

    if (m_state.track_properties[i].mode == Track::ModeYouPlay ||
        m_state.track_properties[i].mode == Track::ModeYouPlaySilently ||
        m_state.track_properties[i].mode == Track::ModeLearning ||
        m_state.track_properties[i].mode == Track::ModeLearningSilently) {
      m_look_ahead_you_play_note_count += m_state.midi->Tracks()[i].Notes().size();
      m_any_you_play_tracks = true;
    }
  }

  // This many microseconds of the song will
  // be shown on the screen at once
  const static microseconds_t DefaultShowDurationMicroseconds = 3250000;
  m_show_duration = DefaultShowDurationMicroseconds;

  m_keyboard = new KeyboardDisplay(KeyboardSize88, GetStateWidth() - Layout::ScreenMarginX*2, CalcKeyboardHeight());

  // Hide the mouse cursor while we're playing
  Compatible::HideMouseCursor();

  ResetSong();
}

PlayingState::~PlayingState() {
  Compatible::ShowMouseCursor();
}

int PlayingState::CalcKeyboardHeight() const {
  // Start with the size of the screen
  int height = GetStateHeight();

  // Allow a couple lines of text below the keys
  height -= 10;

  return height;
}

void PlayingState::Play(microseconds_t delta_microseconds) {

  // Move notes, time tracking, everything
  // delta_microseconds = 0 means, that we are on pause
  MidiEventListWithTrackId evs = m_state.midi->Update(delta_microseconds);

  // These cycle is for keyboard updates (not falling keys)
  const size_t length = evs.size();
  for(size_t i = 0; i < length; ++i) {

    const size_t &track_id = evs[i].first;
    const MidiEvent &ev = evs[i].second;

    // Draw refers to the keys lighting up (automatically) -- not necessarily
    // the falling notes.  The KeyboardDisplay object contains its own logic
    // to decide how to draw the falling notes
    bool draw = false;
    bool play = false;
    switch (m_state.track_properties[track_id].mode) {
    case Track::ModeNotPlayed:           draw = false;  play = false;  break;
    case Track::ModePlayedButHidden:     draw = false;  play = true;   break;
    case Track::ModeYouPlay:             draw = false;  play = false;  break;
    case Track::ModeYouPlaySilently:     draw = false;  play = false;  break;
    case Track::ModeLearning:            draw = false;  play = false;   break;
    case Track::ModeLearningSilently:    draw = false;  play = false;  break;
    case Track::ModePlayedAutomatically: draw = true;   play = true;   break;
    case Track::ModeCount: break;
    }

    // Even in "You Play" tracks, we have to play the non-note
    // events as per usual.
    if (m_state.track_properties[track_id].mode
        && ev.Type() != MidiEventType_NoteOn
        && ev.Type() != MidiEventType_NoteOff)
      play = true;

    if (ev.Type() == MidiEventType_NoteOn || ev.Type() == MidiEventType_NoteOff) {
      int vel = ev.NoteVelocity();
      const string name = MidiEvent::NoteName(ev.NoteNumber());

      bool active = (vel > 0);
      // Display pressed or released a key based on information from a MIDI-file.
      // If this line is deleted, than no notes will be pressed automatically.
      // It is not related to falling notes.
      if (draw)
        m_keyboard->SetKeyActive(name, active, m_state.track_properties[track_id].color);
      filePressedKey(ev.NoteNumber(), active, track_id);
    }

    if (play && m_state.midi_out)
    {
      // Clone event
      MidiEvent ev_out = ev;
      int vel = ev.NoteVelocity();
      // Scale note's volume before playing
      ev_out.SetVelocity(vel * m_state.base_volume);
      m_state.midi_out->Write(ev_out);
    }
  }
}

double PlayingState::CalculateScoreMultiplier() const {
  const static double MaxMultiplier = 5.0;
  double multiplier = 1.0;

  const double combo_addition = m_current_combo / 10.0;
  multiplier += combo_addition;

  return min(MaxMultiplier, multiplier);
}

void PlayingState::Listen() {
  if (!m_state.midi_in)
    return;

  while (m_state.midi_in->KeepReading()) {

    microseconds_t cur_time = m_state.midi->GetSongPositionInMicroseconds();
    MidiEvent ev = m_state.midi_in->Read();
    if (m_state.midi_in->ShouldReconnect())
    {
        m_state.midi_in->Reconnect();
        m_state.midi_out->Reconnect();
        continue;
    }


    // Just eat input if we're paused
    if (m_paused)
      continue;

    // We're only interested in NoteOn and NoteOff
    if (ev.Type() != MidiEventType_NoteOn && ev.Type() != MidiEventType_NoteOff)
      continue;

    // Octave Sliding
    ev.ShiftNote(m_note_offset);

    int note_number = ev.NoteNumber();
    string note_name = MidiEvent::NoteName(note_number);

    // On key release we have to look for existing "active" notes and turn them off.
    if (ev.Type() == MidiEventType_NoteOff || ev.NoteVelocity() == 0) {

      // NOTE: This assumes mono-channel input.  If they're piping an entire MIDI file
      //       (or even the *same* MIDI file) through another source, we could get the
      //       same NoteId on different channels -- and this code would start behaving
      //       incorrectly.
      for (ActiveNoteSet::iterator i = m_active_notes.begin(); i != m_active_notes.end(); ++i) {
        if (ev.NoteNumber() != i->note_id)
          continue;

        // Play it on the correct channel to turn the note we started
        // previously, off.
        ev.SetChannel(i->channel);
        if (m_state.midi_out)
          m_state.midi_out->Write(ev);

        m_active_notes.erase(i);
        break;
      }

      // User releases the key
      // If we delete this line, than all pressed keys will be gray until
      // it is unpressed automatically
      m_keyboard->SetKeyActive(note_name, false, Track::FlatGray);
      userPressedKey(note_number, false);
      continue;
    }

    TranslatedNoteSet::iterator closest_match = m_notes.end();
    for (TranslatedNoteSet::iterator i = m_notes.begin(); i != m_notes.end(); ++i) {

      const microseconds_t window_start = i->start - (KeyboardDisplay::NoteWindowLength / 2);
      const microseconds_t window_end = i->start + (KeyboardDisplay::NoteWindowLength / 2);

      // As soon as we start processing notes that couldn't possibly
      // have been played yet, we're done.
      if (window_start > cur_time)
        break;

      if (i->state != UserPlayable)
        continue;

      if (window_end > cur_time && i->note_id == ev.NoteNumber()) {

        if (closest_match == m_notes.end()) {
          closest_match = i;
          continue;
        }

        microseconds_t this_distance = cur_time - i->start;
        if (i->start > cur_time)
          this_distance = i->start - cur_time;

        microseconds_t known_best = cur_time - closest_match->start;
        if (closest_match->start > cur_time)
          known_best = closest_match->start - cur_time;

        if (this_distance < known_best)
          closest_match = i;
      }
    }

    Track::TrackColor note_color = Track::FlatGray;

    if (closest_match != m_notes.end()) {
      note_color = m_state.track_properties[closest_match->track_id].color;

      // "Open" this note so we can catch the close later and turn off
      // the note.
      ActiveNote n;
      n.channel = closest_match->channel;
      n.note_id = closest_match->note_id;
      n.velocity = closest_match->velocity;
      m_active_notes.insert(n);

      // Play it
      ev.SetChannel(n.channel);
      ev.SetVelocity(n.velocity);

      bool silently =
          m_state.track_properties[closest_match->track_id].mode == Track::ModeYouPlaySilently ||
          m_state.track_properties[closest_match->track_id].mode == Track::ModeLearningSilently;
      if (m_state.midi_out && !silently)
        m_state.midi_out->Write(ev);

      // Adjust our statistics
      const static double NoteValue = 100.0;
      m_state.stats.score += NoteValue * CalculateScoreMultiplier() * (m_state.song_speed / 100.0);

      m_state.stats.notes_user_could_have_played++;
      m_state.stats.speed_integral += m_state.song_speed;

      m_state.stats.notes_user_actually_played++;
      m_current_combo++;
      m_state.stats.longest_combo = max(m_current_combo, m_state.stats.longest_combo);

      TranslatedNote replacement = *closest_match;
      replacement.state = UserHit;

      m_notes.erase(closest_match);
      m_notes.insert(replacement);
    }

    else
      m_state.stats.stray_notes++;

    m_state.stats.total_notes_user_pressed++;
    // Display a pressed key by an user
    // Display a colored key, if it is pressed correctly
    // Otherwise display a grey key
    // 
    // If we comment this code, than a missed user pressed key will not shown.
    // But correct presed key will be shown as usual.
    m_keyboard->SetKeyActive(note_name, true, note_color);
    userPressedKey(note_number, true);
  }
}

void PlayingState::Update() {

  // Calculate how visible the title bar should be
  const static double fade_in_ms = 350.0;
  const static double stay_ms = 2500.0;
  const static double fade_ms = 500.0;

  m_title_alpha = 0.0;
  unsigned long ms = GetStateMilliseconds() * max(m_state.song_speed, 50) / 100;

  if (double(ms) <= stay_ms)
    m_title_alpha = min(1.0, ms / fade_in_ms);

  if (double(ms) >= stay_ms)
    m_title_alpha = min(max((fade_ms - (ms - stay_ms)) / fade_ms, 0.0), 1.0);

  // Lock down the alpha so that if you are slowing the song down as it
  // fades out, it doesn't cut back into a much higher alpha value
  m_title_alpha = min(m_title_alpha, m_max_allowed_title_alpha);

  if (double(ms) > stay_ms)
    m_max_allowed_title_alpha = m_title_alpha;

  microseconds_t delta_microseconds = static_cast<microseconds_t>(GetDeltaMilliseconds()) * 1000;

  // The 100 term is really paired with the playback speed, but this
  // formation is less likely to produce overflow errors.
  delta_microseconds = (delta_microseconds / 100) * m_state.song_speed;

  if (m_paused)
    delta_microseconds = 0;

  // Our delta milliseconds on the first frame after state start is extra
  // long because we just reset the MIDI.  By skipping the "Play" that
  // update, we don't have an artificially fast-forwarded start.
  if (!m_first_update) {
    if (areAllRequiredKeysPressed())
    {
        Play(delta_microseconds);
//      m_should_wait_after_retry = false; // always reset onces pressed
    }
    else
        m_current_combo = 0;

    Listen();
  }

  m_first_update = false;

  microseconds_t cur_time = m_state.midi->GetSongPositionInMicroseconds();

  // Delete notes that are finished playing (and are no longer available to hit)
  TranslatedNoteSet::iterator i = m_notes.begin();
  while (i != m_notes.end()) {
    TranslatedNoteSet::iterator note = i++;

    const microseconds_t window_end = note->start + (KeyboardDisplay::NoteWindowLength / 2);

    if (m_state.midi_in && note->state == UserPlayable && window_end <= cur_time){
      TranslatedNote note_copy = *note;
      note_copy.state = UserMissed;

      m_notes.erase(note);
      m_notes.insert(note_copy);

      // Re-connect the (now-invalid) iterator to the replacement
      note = m_notes.find(note_copy);

      if (m_state.track_properties[note->track_id].is_retry_on
          && !m_should_wait_after_retry)
        // They missed a note and should retry
        // We don't count misses while waiting after retry
        m_should_retry = true;
    }

    if (note->start > cur_time)
      break;

    if (note->end < cur_time && window_end < cur_time) {

      if (note->state == UserMissed) {
        // They missed a note, reset the combo counter
        m_current_combo = 0;

        m_state.stats.notes_user_could_have_played++;
        m_state.stats.speed_integral += m_state.song_speed;
      }

      TranslatedNote history_note = *note;
      m_notes_history.insert(history_note);
      m_notes.erase(note);
    }
  }

  if(IsKeyPressed(KeyGreater))
    m_note_offset += 12;

  if(IsKeyPressed(KeyLess))
    m_note_offset -= 12;

  if (IsKeyPressed(KeyUp)) {
    m_show_duration -= 250000;

    const static microseconds_t MinShowDuration = 250000;
    if (m_show_duration < MinShowDuration)
      m_show_duration = MinShowDuration;
  }

  if (IsKeyPressed(KeyDown)) {
    m_show_duration += 250000;

    const static microseconds_t MaxShowDuration = 10000000;
    if (m_show_duration > MaxShowDuration)
      m_show_duration = MaxShowDuration;
  }

  if (IsKeyPressed(KeyLeft)) {
    m_state.song_speed -= 10;
    if (m_state.song_speed < 0)
      m_state.song_speed = 0;
  }

  if (IsKeyPressed(KeyRight)) {
    m_state.song_speed += 10;
    if (m_state.song_speed > 400)
      m_state.song_speed = 400;
  }

  if (IsKeyPressed(KeyVolumeDown)) {
    m_state.base_volume -= 0.1;
    if (m_state.base_volume < 0)
      m_state.base_volume = 0;
  }

  if (IsKeyPressed(KeyVolumeUp)) {
    m_state.base_volume += 0.1;
    if (m_state.base_volume > 2) // Maximum volume is 200%
      m_state.base_volume = 2;
  }

  if (IsKeyPressed(KeyForward)) {
    // Go 5 seconds forward
    microseconds_t cur_time = m_state.midi->GetSongPositionInMicroseconds();
    microseconds_t new_time = cur_time + 5000000;
    m_state.midi->GoTo(new_time);
    m_required_notes.clear();
    m_state.midi_out->Reset();
    m_keyboard->ResetActiveKeys();
    m_notes = m_state.midi->Notes();
    m_notes_history.clear();
    SetupNoteState();
    m_should_retry = false;
    m_should_wait_after_retry = false;
    m_retry_start = new_time;
  }
  else
  if (IsKeyPressed(KeyBackward)) {
    // Go 5 seconds back
    microseconds_t cur_time = m_state.midi->GetSongPositionInMicroseconds();
    microseconds_t new_time = cur_time - 5000000;
    m_state.midi->GoTo(new_time);
    m_required_notes.clear();
    m_state.midi_out->Reset();
    m_keyboard->ResetActiveKeys();
    m_notes = m_state.midi->Notes();
    m_notes_history.clear();
    SetupNoteState();
    m_should_retry = false;
    m_should_wait_after_retry = false;
    m_retry_start = new_time;
  }
  else
  {
    // Check retry conditions
    // track_properties
    microseconds_t next_bar_time =
        m_state.midi->GetNextBarInMicroseconds(m_retry_start);
    microseconds_t cur_time = m_state.midi->GetSongPositionInMicroseconds();
    // Check point in future
    microseconds_t checkpoint_time = cur_time + delta_microseconds + 1;
//  microseconds_t checkpoint_time = cur_time;
    bool next_bar_exists = next_bar_time != 0;
    bool next_bar_reached = checkpoint_time > next_bar_time;
    if (next_bar_exists && next_bar_reached)
    {
      if (m_should_retry)
      {
        TranslatedNoteSet old = m_notes;
        old.insert(m_notes_history.begin(), m_notes_history.end());

        // Forget failed notes
        m_should_retry = false;
        // Should wait after retry for initial keys to be pressed
        m_should_wait_after_retry = true;

        microseconds_t delta_microseconds = static_cast<microseconds_t>(GetDeltaMilliseconds()) * 1000;
        microseconds_t new_time= m_retry_start-delta_microseconds;
        // Retry
        m_state.midi->GoTo(new_time);
        m_required_notes.clear();
        m_pressed_notes.clear();
        m_state.midi_out->Reset();
        m_keyboard->ResetActiveKeys();
        TranslatedNoteSet def = m_state.midi->Notes();

        // Set retry_state
        // For each current node
        // from SetupNoteState
        m_notes.clear();
        m_notes_history.clear();
        for (TranslatedNoteSet::iterator i = def.begin(); i != def.end(); i++) {
          TranslatedNote n = *i;

          n.state = AutoPlayed;
          n.retry_state = AutoPlayed;
          if (isUserPlayableTrack(n.track_id))
          {
            n.state = UserPlayable;
            n.retry_state = findNodeState(n, old, UserPlayable);
          }

          m_notes.insert(n);
        }

        // To avoid checks for keys that start before and stop after new_time 
        eraseUntilTime(new_time);
      }
      else
      {
        // Handle new retry block
        m_retry_start = cur_time;
      }
    }
  }

  if (IsKeyPressed(KeySpace))
    m_paused = !m_paused;

  if (IsKeyPressed(KeyEscape)) {
    if (m_state.midi_out)
      m_state.midi_out->Reset();

    if (m_state.midi_in)
      m_state.midi_in->Reset();

    ChangeState(new TrackSelectionState(m_state));
    return;
  }

  if (m_state.midi->IsSongOver()) {
    if (m_state.midi_out)
      m_state.midi_out->Reset();

    if (m_state.midi_in)
      m_state.midi_in->Reset();

    if (m_state.midi_in && m_any_you_play_tracks)
      ChangeState(new StatsState(m_state));

    else
      ChangeState(new TrackSelectionState(m_state));

    return;
  }
}

void PlayingState::Draw(Renderer &renderer) const {

  const Tga *key_tex[4] = { GetTexture(PlayKeyRail),
                            GetTexture(PlayKeyShadow),
                            GetTexture(PlayKeysBlack, true),
                            GetTexture(PlayKeysWhite, true) };

  const Tga *note_tex[4] = { GetTexture(PlayNotesWhiteShadow, true),
                             GetTexture(PlayNotesBlackShadow, true),
                             GetTexture(PlayNotesWhiteColor, true),
                             GetTexture(PlayNotesBlackColor, true) };
  renderer.ForceTexture(0);

  // Draw a keyboard, fallen keys and background for them
  m_keyboard->Draw(renderer, key_tex, note_tex, Layout::ScreenMarginX, 0, m_notes, m_show_duration,
                   m_state.midi->GetSongPositionInMicroseconds(), m_state.track_properties,
                   m_state.midi->GetBarLines());

     const int time_pb_width = static_cast<int>(m_state.midi->GetSongPercentageComplete() * (GetStateWidth() - Layout::ScreenMarginX*2));
   //const int pb_x = Layout::ScreenMarginX+8;
   //const int pb_y = CalcKeyboardHeight() - 238;

   const int pb_x = 0;
   const int pb_y = 0;

   renderer.SetColor(0x60, 0x223, 0x60);
   renderer.DrawQuad(pb_x, pb_y, time_pb_width, 6);

  // string title_text = m_state.song_title;

  // double alpha = m_title_alpha;
  // if (m_paused) {
  //   alpha = 1.0;
  //   title_text = "Game Paused";
  // }

  // if (alpha > 0.001) {
  //   renderer.SetColor(0, 0, 0, int(alpha * 160));
  //   renderer.DrawQuad(0, GetStateHeight() / 3, GetStateWidth(), 80);
  //   const Color c = Renderer::ToColor(255, 255, 255, int(alpha * 0xFF));
  //   TextWriter title(GetStateWidth()/2, GetStateHeight()/3 + 25, renderer, true, 24);
  //   title << Text(title_text, c);

  //   // While we're at it, show the key legend
  //   renderer.SetColor(c);
  //   const Tga *keys = GetTexture(PlayKeys);
  //   renderer.DrawTga(keys, GetStateWidth() / 2 - 250, GetStateHeight() / 2);
  // }

  // int text_y = CalcKeyboardHeight() + 42;

  // renderer.SetColor(White);
  // renderer.DrawTga(GetTexture(PlayStatus),  Layout::ScreenMarginX - 1,   text_y);
  // renderer.DrawTga(GetTexture(PlayStatus2), Layout::ScreenMarginX + 273, text_y);

  // string multiplier_text = STRING(fixed << setprecision(1) << CalculateScoreMultiplier() <<
  //                                 " Multiplier");
  // string speed_text = STRING(m_state.song_speed << "% Speed");

  // TextWriter score(Layout::ScreenMarginX + 92, text_y + 5, renderer,
  //                  false, Layout::ScoreFontSize);
  // score << static_cast<int>(m_state.stats.score);

  // TextWriter multipliers(Layout::ScreenMarginX + 232, text_y + 12, renderer,
  //                        false, Layout::TitleFontSize+2);
  // multipliers << Text(multiplier_text, Renderer::ToColor(138, 226, 52));

  // int speed_x_offset = (m_state.song_speed >= 100 ? 0 : 11);
  // TextWriter speed(Layout::ScreenMarginX + 412 + speed_x_offset, text_y + 12,
  //                  renderer, false, Layout::TitleFontSize+2);
  // speed << Text(speed_text, Renderer::ToColor(114, 159, 207));

  // string retry_text = m_should_retry ? "R" : "";

  // TextWriter retry(Layout::ScreenMarginX + 600, text_y + 12,
  //                  renderer, false, Layout::TitleFontSize+2);
  // retry << Text(retry_text, Renderer::ToColor(114, 159, 207));

  // double non_zero_playback_speed = ( (m_state.song_speed == 0) ? 0.1 : (m_state.song_speed/100.0) );
  // microseconds_t tot_seconds = static_cast<microseconds_t>((m_state.midi->GetSongLengthInMicroseconds() /
  //                                                           100000.0) / non_zero_playback_speed);
  // microseconds_t cur_seconds = static_cast<microseconds_t>((m_state.midi->GetSongPositionInMicroseconds() /
  //                                                           100000.0) / non_zero_playback_speed);

  // if (cur_seconds < 0)
  //   cur_seconds = 0;

  // if (cur_seconds > tot_seconds)
  //   cur_seconds = tot_seconds;

  // int completion = static_cast<int>(m_state.midi->GetSongPercentageComplete() * 100.0);

  // unsigned int tot_min = static_cast<unsigned int>((tot_seconds/10) / 60);
  // unsigned int tot_sec = static_cast<unsigned int>((tot_seconds/10) % 60);
  // unsigned int tot_ten = static_cast<unsigned int>( tot_seconds%10);
  // const string total_time = STRING(tot_min << ":" << setfill('0') << setw(2) << tot_sec << "." << tot_ten);

  // unsigned int cur_min = static_cast<unsigned int>((cur_seconds/10) / 60);
  // unsigned int cur_sec = static_cast<unsigned int>((cur_seconds/10) % 60);
  // unsigned int cur_ten = static_cast<unsigned int>( cur_seconds%10      );
  // const string current_time = STRING(cur_min << ":" << setfill('0') << setw(2) << cur_sec << "." << cur_ten);
  // const string percent_complete = STRING(" (" << completion << "%)");

  // text_y += 30 + Layout::SmallFontSize;
  // TextWriter time_text(Layout::ScreenMarginX + 39, text_y+2, renderer, false, Layout::SmallFontSize);
  // time_text << STRING(current_time << " / " << total_time << percent_complete);

  // // Draw a song progress bar along the top of the screen
  // const int time_pb_width = static_cast<int>(m_state.midi->GetSongPercentageComplete() * (GetStateWidth() -
  //                                                                                         Layout::ScreenMarginX*2));
  // const int pb_x = Layout::ScreenMarginX;
  // const int pb_y = CalcKeyboardHeight() + 25;

  // renderer.SetColor(0x50, 0x50, 0x50);
  // renderer.DrawQuad(pb_x, pb_y, time_pb_width, 16);

  // if (m_look_ahead_you_play_note_count > 0) {

  //   const double note_count = 1.0 * m_look_ahead_you_play_note_count;

  //   const int note_miss_pb_width = static_cast<int>(m_state.stats.notes_user_could_have_played /
  //                                                   note_count * (GetStateWidth() - Layout::ScreenMarginX*2));

  //   const int note_hit_pb_width = static_cast<int>(m_state.stats.notes_user_actually_played /
  //                                                  note_count * (GetStateWidth() - Layout::ScreenMarginX*2));

  //   renderer.SetColor(0xCE,0x5C,0x00);
  //   renderer.DrawQuad(pb_x, pb_y - 20, note_miss_pb_width, 16);

  //   renderer.SetColor(0xFC,0xAF,0x3E);
  //   renderer.DrawQuad(pb_x, pb_y - 20, note_hit_pb_width, 16);
  // }

  // // Show the combo
  // if (m_current_combo > 5) {
  //   int combo_font_size = 20;
  //   combo_font_size += (m_current_combo / 10);

  //   int combo_x = GetStateWidth() / 2;
  //   int combo_y = GetStateHeight() - CalcKeyboardHeight() + 30 - (combo_font_size/2);

  //   TextWriter combo_text(combo_x, combo_y, renderer, true, combo_font_size);
  //   combo_text << STRING(m_current_combo << " Combo!");
  // }
}


void PlayingState::userPressedKey(int note_number, bool active)
{
    if (active)
    {
        if (m_should_wait_after_retry)
        {
            m_should_retry = false; // to ensure
            m_should_wait_after_retry = false;
        }
        m_pressed_notes.insert(note_number);
        m_required_notes.erase(note_number);
        m_state.dpms_thread->handleKeyPress();
    }
    else
        m_pressed_notes.erase(note_number);
}

void PlayingState::filePressedKey(int note_number, bool active, size_t track_id)
{
    if (m_state.track_properties[track_id].mode == Track::ModeLearning ||
        m_state.track_properties[track_id].mode == Track::ModeLearningSilently ||
        (m_should_wait_after_retry && isUserPlayableTrack(track_id)))
    {
        if (active)
        {
            m_required_notes.insert(note_number);
        }
        else
            m_required_notes.erase(note_number);
    }
}

bool PlayingState::isKeyPressed(int note_number)
{
    return (m_pressed_notes.find(note_number) != m_pressed_notes.end());
}

bool PlayingState::areAllRequiredKeysPressed()
{
    return m_required_notes.empty();
}

bool PlayingState::isUserPlayableTrack(size_t track_id)
{
  return (m_state.track_properties[track_id].mode == Track::ModeYouPlay ||
          m_state.track_properties[track_id].mode == Track::ModeYouPlaySilently ||
          m_state.track_properties[track_id].mode == Track::ModeLearning ||
          m_state.track_properties[track_id].mode == Track::ModeLearningSilently);
}

void PlayingState::eraseUntilTime(microseconds_t time)
{
  for (TranslatedNoteSet::const_iterator i = m_notes.begin(); i != m_notes.end();) {
    TranslatedNoteSet::const_iterator j = i;
    TranslatedNote n = *i;
    i++;

    // Erase very old notes
    if (n.end < time)
      m_notes.erase(j);
    else
    // Hit still visible once
    if (n.start <= time)
    {
      n.state = UserHit;
      m_notes.erase(j);
      m_notes.insert(n);
    }
  }
}

NoteState PlayingState::findNodeState(const TranslatedNote& note, TranslatedNoteSet& notes, NoteState default_note_state)
{
  // Search by comparing start, end, note_id and track_id
  TranslatedNoteSet::iterator n = notes.find(note);
  if (n == notes.end())
      return default_note_state;

  return n->state;
}
