// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __TRACK_PROPERTIES_H
#define __TRACK_PROPERTIES_H

#include "Renderer.h"

namespace Track {

  // See ModeText in TrackTile.cpp for titles
  // See TractSelectionState.cpp for description
  enum Mode {

    ModePlayedAutomatically,
    ModeYouPlay,
    ModeYouPlaySilently,
    ModeLearning,
    ModeLearningSilently,
    ModePlayedButHidden,
    ModeNotPlayed,

    ModeCount
  };

  // Based on the Open Source icon theme "Tango" color scheme
  // with a few changes.  (e.g. Chameleon NoteBlack is a little
  // darker to distinguish it from NoteWhite, ScarletRed is a
  // little brighter to make it easier on the eyes, etc.)
  const static int ColorCount = 8;
  const static int UserSelectableColorCount = ColorCount - 2;

  enum TrackColor {

    TangoSkyBlue = 0,
    TangoChameleon,
    TangoOrange,
    TangoButter,
    TangoPlum,
    TangoScarletRed,

    FlatGray,
    MissedNote
  };

  const static Color ColorNoteWhite[ColorCount] = {
    { 114, 159, 207, 0xFF },
    { 138, 226,  52, 0xFF },
    { 252, 175,  62, 0xFF },
    { 252, 235,  87, 0xFF },
    { 173, 104, 180, 0xFF },
    { 238,  94,  94, 0xFF },

    {  90,  90,  90, 0xFF },
    {  60,  60,  60, 0xFF }
  };

  const static Color ColorNoteHit[ColorCount] = {
    { 192, 222, 255, 0xFF },
    { 203, 255, 152, 0xFF },
    { 255, 216, 152, 0xFF },
    { 255, 247, 178, 0xFF },
    { 255, 218, 251, 0xFF },
    { 255, 178, 178, 0xFF },

    { 180, 180, 180, 0xFF },
    {  60,  60,  60, 0xFF }
  };

  const static Color ColorNoteBlack[ColorCount] = {
    {  52, 101, 164, 0xFF },
    {  86, 157,  17, 0xFF },
    { 245, 121,   0, 0xFF },
    { 218, 195,   0, 0xFF },
    { 108,  76, 113, 0xFF },
    { 233,  49,  49, 0xFF },

    {  90,  90,  90, 0xFF },
    {  60,  60,  60, 0xFF }
  };

  struct Properties {

    Properties() :
      mode(ModeNotPlayed),
      color(TangoSkyBlue),
      is_retry_on(false) {
    }

    Mode mode;
    TrackColor color;
    bool is_retry_on;
  };

}; // end namespace

#endif // __TRACK_PROPERTIES_H
