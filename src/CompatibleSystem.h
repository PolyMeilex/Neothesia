// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __COMPATIBLE_SYSTEM_H
#define __COMPATIBLE_SYSTEM_H

#include <string>

namespace Compatible {
  
  // Some monotonically increasing value tied to the system
  // clock (but not necessarily based on app-start)
  unsigned long GetMilliseconds();
   
  // Shows an error box with an OK button
  void ShowError(const std::string &err);
  
  int GetDisplayLeft();
  int GetDisplayTop();
  int GetDisplayWidth();
  int GetDisplayHeight();
  
  void HideMouseCursor();
  void ShowMouseCursor();
   
  // Send a message to terminate the application loop gracefully
  void GracefulShutdown();
};

#endif // __COMPATIBLE_SYSTEM_H
