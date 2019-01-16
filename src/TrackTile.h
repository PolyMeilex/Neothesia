// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __TRACK_TILE_H
#define __TRACK_TILE_H

#include "GameState.h"
#include "TextWriter.h"
#include "TrackProperties.h"
#include "MenuLayout.h"

#include "libmidi/Midi.h"

#include <vector>

const int TrackTileWidth = 336;
const int TrackTileHeight = 110;

enum TrackTileGraphic {

  GraphicLeftArrow = 0,
  GraphicRightArrow,
  GraphicColor,
  GraphicPreviewTurnOn,
  GraphicPreviewTurnOff,

  GraphicComputerPlays,
  GraphicHumanPlays,
  GraphicRetryOn,
  GraphicRetryOff,

  Graphic_COUNT
};

class TrackTile {
public:

  TrackTile(int x, int y, size_t track_id,
            Track::TrackColor color, Track::Mode mode,
            bool is_retry_on);

  void Update(const MouseInfo &translated_mouse);
  void Draw(Renderer &renderer, const Midi *midi, Tga *buttons, Tga *box) const;

  int GetX() {
    return m_x;
  }

  int GetY() {
    return m_y;
  }

  Track::Mode GetMode() const {
    return m_mode;
  }

  Track::TrackColor GetColor() const {
    return m_color;
  }

  bool HitPreviewButton() const {
    return button_preview.hit;
  }

  bool IsPreviewOn() const {
    return m_preview_on;
  }

  bool IsRetryOn() const {
    return m_retry_on;
  }

  void TurnOffPreview() {
    m_preview_on = false;
  }

  size_t GetTrackId() const {
    return m_track_id;
  }

  const ButtonState WholeTile() const {
    return whole_tile;
  }

  const ButtonState ButtonPreview() const {
    return button_preview;
  }

  const ButtonState ButtonColor() const {
    return button_color;
  }

  const ButtonState ButtonRetry() const {
    return button_retry;
  }

  const ButtonState ButtonLeft() const {
    return button_mode_left;
  }

  const ButtonState ButtonRight() const {
    return button_mode_right;
  }

private:
  int m_x;
  int m_y;

  Track::Mode m_mode;
  Track::TrackColor m_color;

  bool m_preview_on;
  bool m_retry_on;

  ButtonState whole_tile;
  ButtonState button_preview;
  ButtonState button_color;
  ButtonState button_retry;
  ButtonState button_mode_left;
  ButtonState button_mode_right;

  int LookupGraphic(TrackTileGraphic graphic, bool button_hovering) const;

  // Link to the track index of the Midi object
  size_t m_track_id;
};

#endif // __TRACK_TILE_H
