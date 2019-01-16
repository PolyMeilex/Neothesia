// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __STRING_TILE_H
#define __STRING_TILE_H

#include "GameState.h"
#include "MenuLayout.h"
#include <string>

const int StringTileWidth = 510;
const int StringTileHeight = 80;

class StringTile {
public:

  StringTile(int x, int y, Tga *graphics);

  void Update(const MouseInfo &translated_mouse);
  void Draw(Renderer &renderer) const;

  int GetX() const {
    return m_x;
  }

  int GetY() const {
    return m_y;
  }

  bool Hit() const {
    return whole_tile.hit;
  }

  void SetString(const std::string &s) {
    m_string = s;
  }

  void SetTitle(const std::string &s) {
    m_title = s;
  }

  const ButtonState WholeTile() const {
    return whole_tile;
  }

private:
  int m_x;
  int m_y;

  Tga *m_graphics;

  std::string m_string;
  std::string m_title;

  ButtonState whole_tile;
};

#endif // __STRING_TILE_H
