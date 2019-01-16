// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "MenuLayout.h"
#include "TextWriter.h"
#include "Renderer.h"

using namespace std;

namespace Layout {

  void DrawTitle(Renderer &renderer, const string &title) {

    TextWriter title_writer(ScreenMarginX, ScreenMarginY - TitleFontSize - 10,
                            renderer, false, TitleFontSize);
    title_writer << title;
  }

  void DrawHorizontalRule(Renderer &renderer, int state_width, int y) {

    renderer.SetColor(0x50, 0x50, 0x50);
    renderer.DrawQuad(ScreenMarginX, y - 1, state_width - 2*ScreenMarginX, 3);
  }

  void DrawButton(Renderer &renderer, const ButtonState &button, const Tga *tga) {

    const static Color color = Renderer::ToColor(0xE0,0xE0,0xE0);
    const static Color color_hover = Renderer::ToColor(0xFF,0xFF, 0xFF);

    renderer.SetColor(button.hovering ? color_hover : color);
    renderer.DrawTga(tga, button.x, button.y);
  }


} // End namespace Layout
