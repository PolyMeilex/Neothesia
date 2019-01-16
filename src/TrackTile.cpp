// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "libmidi/Midi.h"

#include "TrackTile.h"
#include "Renderer.h"
#include "Tga.h"

const static int GraphicWidth = 36;
const static int GraphicHeight = 36;

// Only used here
const static char* ModeText[Track::ModeCount] = {
  "Played Automatically",
  "You Play", // You Press, we help
  "You Play Silently", // You Play
  "Learning",
  "Learning Silently",
  "Played But Hidden",
  "Not Played"
};

TrackTile::TrackTile(int x, int y, size_t track_id, Track::TrackColor color, Track::Mode mode,
                     bool is_retry_on) :
  m_x(x),
  m_y(y),
  m_mode(mode),
  m_color(color),
  m_preview_on(false),
  m_retry_on(is_retry_on),
  m_track_id(track_id) {

  // Initialize the size and position of each button
  whole_tile = ButtonState(0, 0, TrackTileWidth, TrackTileHeight);
  button_mode_left  = ButtonState(  2, 68, GraphicWidth, GraphicHeight);
  button_mode_right = ButtonState(192, 68, GraphicWidth, GraphicHeight);
  button_color      = ButtonState(228, 68, GraphicWidth, GraphicHeight);
  button_preview    = ButtonState(264, 68, GraphicWidth, GraphicHeight);
  button_retry      = ButtonState(300, 68, GraphicWidth, GraphicHeight);
}

void TrackTile::Update(const MouseInfo &translated_mouse) {

  // Update the mouse state of each button
  whole_tile.Update(translated_mouse);
  button_preview.Update(translated_mouse);
  button_retry.Update(translated_mouse);
  button_color.Update(translated_mouse);
  button_mode_left.Update(translated_mouse);
  button_mode_right.Update(translated_mouse);

  if (button_mode_left.hit) {

    int mode = static_cast<int>(m_mode) - 1;
    if (mode < 0)
      mode = Track::ModeCount - 1;

    m_mode = static_cast<Track::Mode>(mode);
  }

  if (button_mode_right.hit) {

    int mode = static_cast<int>(m_mode) + 1;
    if (mode >= Track::ModeCount)
      mode = 0;

    m_mode = static_cast<Track::Mode>(mode);
  }

  if (button_preview.hit)
    m_preview_on = !m_preview_on;

  if (button_retry.hit)
    m_retry_on = !m_retry_on;

  if (button_color.hit && m_mode != Track::ModeNotPlayed && m_mode != Track::ModePlayedButHidden) {
    int color = static_cast<int>(m_color) + 1;
    if (color >= Track::UserSelectableColorCount)
      color = 0;

    m_color = static_cast<Track::TrackColor>(color);
  }

}

int TrackTile::LookupGraphic(TrackTileGraphic graphic, bool button_hovering) const {

  // There are three sets of graphics
  // set 0: window lit, hovering
  // set 1: window lit, not-hovering
  // set 2: window unlit, (implied not-hovering)
  int graphic_set = 2;
  if (whole_tile.hovering)
    graphic_set--;

  if (button_hovering)
    graphic_set--;

  const int set_offset = GraphicWidth * Graphic_COUNT;
  const int graphic_offset = GraphicWidth * graphic;

  return (set_offset * graphic_set) + graphic_offset;
}

void TrackTile::Draw(Renderer &renderer, const Midi *midi, Tga *buttons, Tga *box) const {

  const MidiTrack &track = midi->Tracks()[m_track_id];

  bool gray_out_buttons = false;
  Color light  = Track::ColorNoteWhite[m_color];
  Color medium = Track::ColorNoteBlack[m_color];

  if (m_mode == Track::ModePlayedButHidden || m_mode == Track::ModeNotPlayed) {

    gray_out_buttons = true;
    light  = Renderer::ToColor(0xB0,0xB0,0xB0);
    medium = Renderer::ToColor(0x70,0x70,0x70);
  }

  Color color_tile = medium;
  Color color_tile_hovered = light;

  renderer.SetOffset(m_x, m_y);

  renderer.SetColor(whole_tile.hovering ? color_tile_hovered : color_tile);
  renderer.DrawTga(box, -10, -6);

  renderer.SetColor(White);

  // Write song info to the tile
  TextWriter instrument(95, 14, renderer, false, 14);
  instrument << track.InstrumentName();
  TextWriter note_count(95, 35, renderer, false, 14);
  note_count << static_cast<const unsigned int>(track.Notes().size());

  int color_offset = GraphicHeight * static_cast<int>(m_color);
  if (gray_out_buttons)
    color_offset = GraphicHeight * Track::UserSelectableColorCount;

  renderer.DrawTga(buttons, BUTTON_RECT(button_mode_left),
                   LookupGraphic(GraphicLeftArrow, button_mode_left.hovering),
                   color_offset);

  renderer.DrawTga(buttons, BUTTON_RECT(button_mode_right),
                   LookupGraphic(GraphicRightArrow, button_mode_right.hovering),
                   color_offset);

  renderer.DrawTga(buttons, BUTTON_RECT(button_color),
                   LookupGraphic(GraphicColor, button_color.hovering),
                   color_offset);

  TrackTileGraphic preview_graphic = GraphicPreviewTurnOn;
  if (m_preview_on)
    preview_graphic = GraphicPreviewTurnOff;

  TrackTileGraphic retry_graphic = GraphicRetryOff;
  if (m_retry_on)
    retry_graphic = GraphicRetryOn;

  renderer.DrawTga(buttons, BUTTON_RECT(button_preview),
                   LookupGraphic(preview_graphic, button_preview.hovering),
                   color_offset);

  renderer.DrawTga(buttons, BUTTON_RECT(button_retry),
                   LookupGraphic(retry_graphic, button_retry.hovering),
                   color_offset);

  // Draw mode text
  TextWriter mode(42, 81, renderer, false, 12);
  mode << ModeText[m_mode];

  renderer.ResetOffset();
}

