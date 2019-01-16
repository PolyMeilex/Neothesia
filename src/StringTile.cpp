// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "StringTile.h"
#include "TextWriter.h"
#include "Renderer.h"

StringTile::StringTile(int x, int y, Tga *graphics) :
  m_x(x), m_y(y), m_graphics(graphics) {

  whole_tile = ButtonState(0, 0, StringTileWidth, StringTileHeight);
}

void StringTile::Update(const MouseInfo &translated_mouse) {

  whole_tile.Update(translated_mouse);
}

void StringTile::Draw(Renderer &renderer) const {

  renderer.SetOffset(m_x, m_y);

  const Color hover = Renderer::ToColor(0xFF, 0xFF, 0xFF);
  const Color no_hover = Renderer::ToColor(0xE0, 0xE0, 0xE0);
  renderer.SetColor(whole_tile.hovering ? hover : no_hover);
  renderer.DrawTga(m_graphics, 0, 0);

  TextWriter text(20, 46, renderer, false, 14);
  text << m_string;

  renderer.ResetOffset();
}

