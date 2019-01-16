// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __FRAME_COUNTER_H
#define __FRAME_COUNTER_H

class FrameCounter {
public:

  // averaged_over_milliseconds is the length of time GetFramesPerSecond
  // should average the frame count over in order to smooth the rate.
  FrameCounter(double averaged_over_milliseconds) :
    m_average_over_ms(averaged_over_milliseconds),
    m_period_ms(0),
    m_frames(0),
    m_cached_fps(0) {

    if (m_average_over_ms <= 50.0) m_average_over_ms = 50.0;
  }

  void Frame(double delta_ms) {

    if (delta_ms < 0.0)
      return;

    m_period_ms += delta_ms;
    m_frames++;

    if (m_period_ms > m_average_over_ms) {
      m_cached_fps = static_cast<double>(m_frames) / m_period_ms * 1000.0;

      m_frames = 0;
      m_period_ms = 0;
    }
  }

  double GetFramesPerSecond() const {
    return m_cached_fps;
  }

private:
  double m_average_over_ms;
  double m_period_ms;
  int m_frames;

  double m_cached_fps;
};

#endif // __FRAME_COUNTER_H
