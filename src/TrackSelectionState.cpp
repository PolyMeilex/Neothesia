// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "TrackSelectionState.h"

#include "TitleState.h"
#include "PlayingState.h"
#include "MenuLayout.h"
#include "Renderer.h"
#include "Textures.h"

#include "libmidi/Midi.h"
#include "libmidi/MidiUtil.h"
#include "MidiComm.h"

using namespace std;

TrackSelectionState::TrackSelectionState(const SharedState &state) :
  m_page_count(0),
  m_current_page(0),
  m_tiles_per_page(0),
  m_preview_on(false),
  m_first_update_after_seek(false),
  m_preview_track_id(0),
  m_state(state) {
}

void TrackSelectionState::Init() {

  if (m_state.midi_out)
    m_state.midi_out->Reset();

  Midi &m = *m_state.midi;

  // Prepare a very simple count of the playable tracks first
  int track_count = 0;
  for (size_t i = 0; i < m.Tracks().size(); ++i) {
    if (m.Tracks()[i].Notes().size())
      track_count++;
  }

  m_back_button = ButtonState(
    Layout::ScreenMarginX,
    GetStateHeight() - Layout::ScreenMarginY/2 - Layout::ButtonHeight/2,
    Layout::ButtonWidth, Layout::ButtonHeight);

  m_continue_button = ButtonState(
    GetStateWidth() - Layout::ScreenMarginX - Layout::ButtonWidth,
    GetStateHeight() - Layout::ScreenMarginY/2 - Layout::ButtonHeight/2,
    Layout::ButtonWidth, Layout::ButtonHeight);

  // Determine how many track tiles we can fit
  // horizontally and vertically. Integer division
  // helps us round down here.
  int tiles_across = (GetStateWidth() + Layout::ScreenMarginX) /
    (TrackTileWidth + Layout::ScreenMarginX);

  tiles_across = max(tiles_across, 1);

  int tiles_down = (GetStateHeight() - Layout::ScreenMarginX - Layout::ScreenMarginY * 2) /
    (TrackTileHeight + Layout::ScreenMarginX);

  tiles_down = max(tiles_down, 1);

  // Calculate how many pages of tracks there will be
  m_tiles_per_page = tiles_across * tiles_down;

  m_page_count        = track_count / m_tiles_per_page;
  const int remainder = track_count % m_tiles_per_page;
  if (remainder > 0)
    m_page_count++;

  // If we have fewer than one row of tracks, just
  // center the tracks we do have
  if (track_count < tiles_across)
    tiles_across = track_count;

  // Determine how wide that many track tiles will
  // actually be, so we can center the list
  int all_tile_widths = tiles_across * TrackTileWidth + (tiles_across-1) * Layout::ScreenMarginX;
  int global_x_offset = (GetStateWidth() - all_tile_widths) / 2;

  const static int starting_y = 100;

  int tiles_on_this_line = 0;
  int tiles_on_this_page = 0;
  int current_y = starting_y;

  for (size_t i = 0; i < m.Tracks().size(); ++i) {

    const MidiTrack &t = m.Tracks()[i];
    if (t.Notes().size() == 0)
      continue;

    int x = global_x_offset + (TrackTileWidth + Layout::ScreenMarginX)*tiles_on_this_line;
    int y = current_y;

    Track::Mode mode = Track::ModePlayedAutomatically;
    bool is_retry_on = false;
    if (t.IsPercussion())
      mode = Track::ModePlayedButHidden;

    Track::TrackColor color = static_cast<Track::TrackColor>((m_track_tiles.size()) % Track::UserSelectableColorCount);

    // If we came back here from StatePlaying, reload all our preferences
    if (m_state.track_properties.size() > i) {

      color = m_state.track_properties[i].color;
      mode = m_state.track_properties[i].mode;
      is_retry_on = m_state.track_properties[i].is_retry_on;
    }

    TrackTile tile(x, y, i, color, mode, is_retry_on);

    m_track_tiles.push_back(tile);


    tiles_on_this_line++;
    tiles_on_this_line %= tiles_across;

    if (!tiles_on_this_line)
      current_y += TrackTileHeight + Layout::ScreenMarginX;


    tiles_on_this_page++;
    tiles_on_this_page %= m_tiles_per_page;

    if (!tiles_on_this_page) {
      current_y = starting_y;
      tiles_on_this_line = 0;
    }
  }
}

vector<Track::Properties> TrackSelectionState::BuildTrackProperties() const {

  vector<Track::Properties> props;
  for (size_t i = 0; i < m_state.midi->Tracks().size(); ++i) {
    props.push_back(Track::Properties());
  }

  // Populate it with the tracks that have notes
  for (vector<TrackTile>::const_iterator i = m_track_tiles.begin(); i != m_track_tiles.end(); ++i) {
    props[i->GetTrackId()].color = i->GetColor();
    props[i->GetTrackId()].mode = i->GetMode();
    props[i->GetTrackId()].is_retry_on = i->IsRetryOn();
  }

  return props;
}

void TrackSelectionState::Update() {
  m_continue_button.Update(MouseInfo(Mouse()));
  m_back_button.Update(MouseInfo(Mouse()));

  if (IsKeyPressed(KeyEscape) || m_back_button.hit) {

    if (m_state.midi_out)
      m_state.midi_out->Reset();

    m_state.track_properties = BuildTrackProperties();
    ChangeState(new TitleState(m_state));
    return;
  }

  if (IsKeyPressed(KeyEnter) || m_continue_button.hit) {

    if (m_state.midi_out)
      m_state.midi_out->Reset();

    m_state.track_properties = BuildTrackProperties();
    ChangeState(new PlayingState(m_state));

    return;
  }

  if (IsKeyPressed(KeyDown) || IsKeyPressed(KeyRight)) {
    m_current_page++;

    if (m_current_page == m_page_count)
      m_current_page = 0;
  }

  if (IsKeyPressed(KeyUp) || IsKeyPressed(KeyLeft)) {
    m_current_page--;

    if (m_current_page < 0)
      m_current_page += m_page_count;
  }

  m_tooltip = "";

  if (m_back_button.hovering)
    m_tooltip = "Click to return to the title screen.";

  if (m_continue_button.hovering)
    m_tooltip = "Click to begin playing with these settings.";

  // Our delta milliseconds on the first frame after we seek down to the
  // first note is extra long because the seek takes a while.  By skipping
  // the "Play" that update, we don't have an artificially fast-forwarded
  // start.
  if (!m_first_update_after_seek)
    PlayTrackPreview(static_cast<microseconds_t>(GetDeltaMilliseconds()) * 1000);

  m_first_update_after_seek = false;

  // Do hit testing on each tile button on this page
  size_t start = m_current_page * m_tiles_per_page;
  size_t end = min( static_cast<size_t>((m_current_page+1) * m_tiles_per_page), m_track_tiles.size() );
  for (size_t i = start; i < end; ++i) {

    TrackTile &t = m_track_tiles[i];

    MouseInfo mouse = MouseInfo(Mouse());
    mouse.x -= t.GetX();
    mouse.y -= t.GetY();

    t.Update(mouse);

    if (t.ButtonLeft().hovering || t.ButtonRight().hovering) {

      switch (t.GetMode()) {
      case Track::ModeNotPlayed:
        m_tooltip = "Track won't be played or shown during the game.";
        break;

      case Track::ModePlayedAutomatically:
        m_tooltip = "Track will be played automatically by the game.";
        break;

      case Track::ModePlayedButHidden:
        m_tooltip = "Track will be played automatically by the game, but also hidden from view.";
        break;

      case Track::ModeYouPlay:
        m_tooltip = "'You Play' means you want to play this track yourself.";
        break;

      case Track::ModeYouPlaySilently:
        m_tooltip = "Same as 'You Play', ignore velocity from MIDI.";
        break;

      case Track::ModeLearning:
        m_tooltip = "Wait for you to play.";
        break;

      case Track::ModeLearningSilently:
        m_tooltip = "Wait for you to play, do not produce sounds from MIDI.";
        break;

      case Track::ModeCount:
        break;
      }
    }

    if (t.ButtonPreview().hovering) {
      if (t.IsPreviewOn())
        m_tooltip = "Turn track preview off.";

      else
        m_tooltip = "Preview how this track sounds.";
    }

    if (t.ButtonColor().hovering)
      m_tooltip = "Pick a color for this track's notes.";

    if (t.ButtonRetry().hovering) {
      if (t.IsRetryOn())
        m_tooltip = "Ignore failed tempo blocks.";

      else
        m_tooltip = "Repeat failed tempo blocks.";
    }

    if (t.HitPreviewButton()) {

      if (m_state.midi_out)
        m_state.midi_out->Reset();

      if (t.IsPreviewOn()) {

        // Turn off any other preview modes
        for (size_t j = 0; j < m_track_tiles.size(); ++j) {
          if (i == j)
            continue;

          m_track_tiles[j].TurnOffPreview();
        }

        const microseconds_t PreviewLeadIn  = 25000;
        const microseconds_t PreviewLeadOut = 25000;

        m_preview_on = true;
        m_preview_track_id = t.GetTrackId();
        m_state.midi->Reset(PreviewLeadIn, PreviewLeadOut);
        PlayTrackPreview(0);

        // Find the first note in this track so we can skip right to the good part.
        microseconds_t additional_time = -PreviewLeadIn;
        const MidiTrack &track = m_state.midi->Tracks()[m_preview_track_id];
        for (size_t i = 0; i < track.Events().size(); ++i) {

          const MidiEvent &ev = track.Events()[i];
          if (ev.Type() == MidiEventType_NoteOn && ev.NoteVelocity() > 0) {
            additional_time += track.EventUsecs()[i] - m_state.midi->GetDeadAirStartOffsetMicroseconds() - 1;
            break;
          }
        }

        PlayTrackPreview(additional_time);
        m_first_update_after_seek = true;
      }

      else
        m_preview_on = false;
    }
  }
}

void TrackSelectionState::PlayTrackPreview(microseconds_t delta_microseconds) {

  if (!m_preview_on)
    return;

  MidiEventListWithTrackId evs = m_state.midi->Update(delta_microseconds);

  for (MidiEventListWithTrackId::const_iterator i = evs.begin(); i != evs.end(); ++i) {
    const MidiEvent &ev = i->second;

    if (i->first != m_preview_track_id)
      continue;

    if (m_state.midi_out)
      m_state.midi_out->Write(ev);
  }
}

void TrackSelectionState::Draw(Renderer &renderer) const {

  Layout::DrawTitle(renderer, "Choose Tracks To Play");

  Layout::DrawHorizontalRule(renderer, GetStateWidth(), Layout::ScreenMarginY);
  Layout::DrawHorizontalRule(renderer, GetStateWidth(), GetStateHeight() - Layout::ScreenMarginY);

  Layout::DrawButton(renderer, m_continue_button, GetTexture(ButtonPlaySong));
  Layout::DrawButton(renderer, m_back_button, GetTexture(ButtonBackToTitle));

  // Write our page count on the screen
  TextWriter pagination(GetStateWidth()/2, GetStateHeight() - Layout::SmallFontSize - 30,
                        renderer, true, Layout::ButtonFontSize);

  pagination << Text(STRING("Page " << (m_current_page+1) << " of " <<
                            m_page_count << " (arrow keys change page)"), Gray);

  TextWriter tooltip(GetStateWidth()/2, GetStateHeight() - Layout::SmallFontSize - 54,
                     renderer, true, Layout::ButtonFontSize);

  tooltip << m_tooltip;

  Tga *buttons = GetTexture(InterfaceButtons);
  Tga *box = GetTexture(TrackPanel);

  // Draw each track tile on the current page
  size_t start = m_current_page * m_tiles_per_page;
  size_t end = min(static_cast<size_t>((m_current_page+1) * m_tiles_per_page), m_track_tiles.size());

  for (size_t i = start; i < end; ++i) {
    m_track_tiles[i].Draw(renderer, m_state.midi, buttons, box);
  }
}
