// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __STATS_STATE_H
#define __STATS_STATE_H

#include "SharedState.h"
#include "GameState.h"
#include "MenuLayout.h"

class StatsState : public GameState {
public:

  StatsState(const SharedState &state) :
    m_state(state) {
  }

protected:
  virtual void Init();
  virtual void Update();
  virtual void Draw(Renderer &renderer) const;

private:
  ButtonState m_continue_button;
  ButtonState m_back_button;

  std::string m_tooltip;

  SharedState m_state;
};

#endif // __STATS_STATE_H
