// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "StatsState.h"
#include "TrackSelectionState.h"
#include "PlayingState.h"
#include "Renderer.h"
#include "Textures.h"

#include <iomanip>

using namespace std;

void StatsState::Init() {
  m_back_button = ButtonState(
      Layout::ScreenMarginX,
      GetStateHeight() - Layout::ScreenMarginY/2 - Layout::ButtonHeight/2,
      Layout::ButtonWidth, Layout::ButtonHeight);

  m_continue_button = ButtonState(
      GetStateWidth() - Layout::ScreenMarginX - Layout::ButtonWidth,
      GetStateHeight() - Layout::ScreenMarginY/2 - Layout::ButtonHeight/2,
      Layout::ButtonWidth, Layout::ButtonHeight);
}

void StatsState::Update() {

  MouseInfo mouse = Mouse();

  m_continue_button.Update(mouse);
  m_back_button.Update(mouse);

  if (IsKeyPressed(KeyEscape) || m_back_button.hit) {
    ChangeState(new TrackSelectionState(m_state));
    return;
  }

  if (IsKeyPressed(KeyEnter) || m_continue_button.hit) {
    ChangeState(new PlayingState(m_state));
    return;
  }

  m_tooltip = "";
  if (m_back_button.hovering)
    m_tooltip = "Return to the track selection screen.";

  if (m_continue_button.hovering)
    m_tooltip = "Try this song again with the same settings.";
}

void StatsState::Draw(Renderer &renderer) const {

  const bool ConstrainedHeight = (GetStateHeight() < 720);

  int left = GetStateWidth() / 2 + 40;
  const int InstructionsY = ConstrainedHeight ? 120 : 263;

  renderer.SetColor(White);
  renderer.DrawTga(GetTexture(StatsText), left - 270, InstructionsY - 113);

  if (!ConstrainedHeight) {
    Layout::DrawTitle(renderer, m_state.song_title);
    Layout::DrawHorizontalRule(renderer, GetStateWidth(), Layout::ScreenMarginY);
  }

  Layout::DrawHorizontalRule(renderer, GetStateWidth(), GetStateHeight() - Layout::ScreenMarginY);

  Layout::DrawButton(renderer, m_continue_button, GetTexture(ButtonRetrySong));
  Layout::DrawButton(renderer, m_back_button, GetTexture(ButtonChooseTracks));

  const SongStatistics &s = m_state.stats;

  double hit_percent = 0.0;
  if (s.notes_user_could_have_played > 0) {
    hit_percent = 100.0 * (s.notes_user_actually_played / (s.notes_user_could_have_played * 1.0));
  }

  string grade = "F";
  if (hit_percent >= 50) grade = "D-";
  if (hit_percent >= 55) grade = "D";
  if (hit_percent >= 63) grade = "D+";
  if (hit_percent >= 70) grade = "C-";
  if (hit_percent >= 73) grade = "C";
  if (hit_percent >= 77) grade = "C+";
  if (hit_percent >= 80) grade = "B-";
  if (hit_percent >= 83) grade = "B";
  if (hit_percent >= 87) grade = "B+";
  if (hit_percent >= 90) grade = "A-";
  if (hit_percent >= 93) grade = "A";
  if (hit_percent >= 97) grade = "A+";
  if (hit_percent >= 99) grade = "A++";
  if (hit_percent >= 100) grade = "A+++";

  int stray_percent = 0;
  if (s.total_notes_user_pressed > 0)
    stray_percent = static_cast<int>((100.0 * s.stray_notes) / s.total_notes_user_pressed);

  int average_speed = 0;
  if (s.notes_user_could_have_played > 0)
    average_speed = s.speed_integral / s.notes_user_could_have_played;

   // Choose a dynamic color for the grade
  const double p = hit_percent / 100.0;
  const double r = max(0.0, 1 - (p*p*p*p));
  const double g = max(0.0, 1 - (((p-  1)*4)*((p-  1)*4)));
  const double b = max(0.0, 1 - (((p-.75)*5)*((p-.75)*5)));

  const Color c = Renderer::ToColor(int(r*0xFF), int(g*0xFF), int(b*0xFF));

  TextWriter grade_text(left - 5, InstructionsY - 15, renderer, false, 100);
  grade_text << Text(grade, c);

  TextWriter score(left, InstructionsY + 112, renderer, false, 28);
  score << STRING(static_cast<int>(s.score));

  TextWriter speed(left, InstructionsY + 147, renderer, false, 28);
  speed << STRING(average_speed << " %");

  TextWriter good(left, InstructionsY + 218, renderer, false, 28);
  good << STRING(s.notes_user_actually_played << " / " <<
                 s.notes_user_could_have_played << "  (" <<
                 static_cast<int>(hit_percent) << " %" << ")");

  TextWriter stray(left, InstructionsY + 255, renderer, false, 28);
  stray << STRING(s.stray_notes << "  (" << stray_percent << " %" << ")");

  TextWriter combo(left, InstructionsY + 323, renderer, false, 28);
  combo << STRING(s.longest_combo);

  TextWriter tooltip(GetStateWidth() / 2,
                     GetStateHeight() - Layout::ScreenMarginY/2 - Layout::TitleFontSize/2,
                     renderer, true, Layout::TitleFontSize);

  tooltip << m_tooltip;
}
