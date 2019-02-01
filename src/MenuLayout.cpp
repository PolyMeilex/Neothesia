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

  void DrawButton(Renderer &renderer, const ButtonState &button, const string Title) {

    const static Color color = Renderer::ToColor(32,32,32);
    const static Color color_hover = Renderer::ToColor(42,42,42);

    renderer.SetColor(button.hovering ? color_hover : color);
    // renderer.DrawTga(tga, button.x, button.y);
    renderer.ForceTexture(0);
    renderer.DrawQuad(button.x,button.y,button.w,button.h);

    TextWriter title_writer(button.x + button.w/2 - 10, button.y + button.h/2 - 5, renderer, true, 10);
    title_writer << Title;
  }


} // End namespace Layout
