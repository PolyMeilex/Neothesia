// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <sys/time.h>
#include <gtkmm.h>

#include "MidiComm.h"
#include "CompatibleSystem.h"
#include "StringUtil.h"
#include "Version.h"

using namespace std;

namespace Compatible {

  unsigned long GetMilliseconds() {

    timeval tv;
    gettimeofday(&tv, 0);
    return (tv.tv_sec * 1000) + (tv.tv_usec / 1000);
  }


  void ShowError(const string &err) {

    const static string friendly_app_name =
      STRING("Neothesia " << NeothesiaVersionString);
    const static string message_box_title =
      STRING(friendly_app_name << " Error");

    Gtk::MessageDialog dialog(err, false, Gtk::MESSAGE_ERROR);
    dialog.run();
  }

  void HideMouseCursor() {
    // TODO
  }

  void ShowMouseCursor() {
    // TODO
  }

  void GetDisplayRect(Gdk::Rectangle &rect) {
	  static bool inited = false;
	  static Gdk::Rectangle monitor_geometry;

	  if (!inited) {
		  auto display = Gdk::Display::get_default();

		  int pointer_x, pointer_y;
		  Gdk::ModifierType pointer_mask;
		  display->get_pointer(pointer_x, pointer_y, pointer_mask);

		  auto screen = display->get_default_screen();

		  screen->get_monitor_geometry(
				  screen->get_monitor_at_point(pointer_x, pointer_y),
				  monitor_geometry
		  );
		  inited = true;
	  }
	  rect = monitor_geometry;
  }

  int GetDisplayLeft() {
		Gdk::Rectangle rect;
		GetDisplayRect(rect);
	    return rect.get_x();
  }

  int GetDisplayTop() {
		Gdk::Rectangle rect;
		GetDisplayRect(rect);
	    return rect.get_y();
  }

  int GetDisplayWidth() {
	Gdk::Rectangle rect;
	GetDisplayRect(rect);
    return rect.get_width();
  }

  int GetDisplayHeight() {
		Gdk::Rectangle rect;
		GetDisplayRect(rect);
	    return rect.get_height();
  }

  void GracefulShutdown() {
    midiStop();
    Gtk::Main::instance()->quit();
  }

}; // End namespace
