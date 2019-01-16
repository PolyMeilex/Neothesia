// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <string>

#include "OSGraphics.h"
#include "StringUtil.h"
#include "FileSelector.h"
#include "UserSettings.h"
#include "Version.h"
#include "CompatibleSystem.h"
#include "LinthesiaError.h"
#include "Tga.h"
#include "Renderer.h"
#include "SharedState.h"
#include "GameState.h"
#include "TitleState.h"
#include "DpmsThread.h"

#include "libmidi/Midi.h"
#include "libmidi/MidiUtil.h"
#include <gconfmm.h>

#ifndef GRAPHDIR
#define GRAPHDIR "../graphics"
#endif

using namespace std;

GameStateManager* state_manager;

const static string application_name = "Neothesia";
const static string friendly_app_name = STRING("Neothesia " <<
					       NeothesiaVersionString);

const static string error_header1 = "Neothesia detected a";
const static string error_header2 = " problem and must close:\n\n";
const static string error_footer = "\n\n Error )-: ";

class EdgeTracker  {
public:

  EdgeTracker() :
    active(true),
    just_active(true) {
  }

  void Activate() {
    just_active = true;
    active = true;
  }

  void Deactivate() {
    just_active = false;
    active = false;
  }

  bool IsActive() {
    return active;
  }

  bool JustActivated() {
    bool was_active = just_active;
    just_active = false;
    return was_active;
  }

private:
  bool active;
  bool just_active;
};

static EdgeTracker window_state;

class DrawingArea : public Gtk::GL::DrawingArea {
public:

  DrawingArea(const Glib::RefPtr<const Gdk::GL::Config>& config) :
    Gtk::GL::DrawingArea(config) {

    set_events(Gdk::POINTER_MOTION_MASK |
               Gdk::BUTTON_PRESS_MASK   |
               Gdk::BUTTON_RELEASE_MASK |
               Gdk::KEY_PRESS_MASK      |
               Gdk::KEY_RELEASE_MASK);

    set_can_focus();

    signal_motion_notify_event().connect(sigc::mem_fun(*this, &DrawingArea::on_motion_notify));
    signal_button_press_event().connect(sigc::mem_fun(*this, &DrawingArea::on_button_press));
    signal_button_release_event().connect(sigc::mem_fun(*this, &DrawingArea::on_button_press));
    signal_key_press_event().connect(sigc::mem_fun(*this, &DrawingArea::on_key_press));
    signal_key_release_event().connect(sigc::mem_fun(*this, &DrawingArea::on_key_release));
  }

  bool GameLoop();

protected:
  virtual void on_realize();
  virtual bool on_configure_event(GdkEventConfigure* event);
  virtual bool on_expose_event(GdkEventExpose* event);

  virtual bool on_motion_notify(GdkEventMotion* event);
  virtual bool on_button_press(GdkEventButton* event);
  virtual bool on_key_press(GdkEventKey* event);
  virtual bool on_key_release(GdkEventKey* event);
};

bool DrawingArea::on_motion_notify(GdkEventMotion* event) {

  state_manager->MouseMove(event->x, event->y);
  return true;
}

bool DrawingArea::on_button_press(GdkEventButton* event) {

  MouseButton b;

  // left and right click allowed
  if (event->button == 1)
    b = MouseLeft;
  else if (event->button == 3)
    b = MouseRight;

  // ignore other buttons
  else
    return false;

  // press or release?
  if (event->type == GDK_BUTTON_PRESS)
    state_manager->MousePress(b);
  else if (event->type == GDK_BUTTON_RELEASE)
    state_manager->MouseRelease(b);
  else
    return false;

  return true;
}

// FIXME: use user settings to do this mapping
int keyToNote(GdkEventKey* event) {
  const unsigned short oct = 4;

  switch(event->keyval) {
  /* no key for C :( */
  case GDK_masculine:  return 12*oct + 1;      /* C# */
  case GDK_Tab:        return 12*oct + 2;      /* D  */
  case GDK_1:          return 12*oct + 3;      /* D# */
  case GDK_q:          return 12*oct + 4;      /* E  */
  case GDK_w:          return 12*oct + 5;      /* F  */
  case GDK_3:          return 12*oct + 6;      /* F# */
  case GDK_e:          return 12*oct + 7;      /* G  */
  case GDK_4:          return 12*oct + 8;      /* G# */
  case GDK_r:          return 12*oct + 9;      /* A  */
  case GDK_5:          return 12*oct + 10;     /* A# */
  case GDK_t:          return 12*oct + 11;     /* B  */

  case GDK_y:          return 12*(oct+1) + 0;  /* C  */
  case GDK_7:          return 12*(oct+1) + 1;  /* C# */
  case GDK_u:          return 12*(oct+1) + 2;  /* D  */
  case GDK_8:          return 12*(oct+1) + 3;  /* D# */
  case GDK_i:          return 12*(oct+1) + 4;  /* E  */
  case GDK_o:          return 12*(oct+1) + 5;  /* F  */
  case GDK_0:          return 12*(oct+1) + 6;  /* F# */
  case GDK_p:          return 12*(oct+1) + 7;  /* G  */
  case GDK_apostrophe: return 12*(oct+1) + 8;  /* G# */
  case GDK_dead_grave: return 12*(oct+1) + 9;  /* A  */
  case GDK_exclamdown: return 12*(oct+1) + 10; /* A# */
  case GDK_plus:       return 12*(oct+1) + 11; /* B  */
  }

  return -1;
}

typedef map<int,sigc::connection> ConnectMap;
ConnectMap pressed;

bool __sendNoteOff(int note) {

  ConnectMap::iterator it = pressed.find(note);
  if (it == pressed.end())
    return false;

  sendNote(note, false);
  pressed.erase(it);

  return true;
}

bool DrawingArea::on_key_press(GdkEventKey* event) {

  // if is a note...
  int note = keyToNote(event);
  if (note >= 0) {

    // if first press, send Note-On
    ConnectMap::iterator it = pressed.find(note);
    if (it == pressed.end())
      sendNote(note, true);

    // otherwise, cancel emission of Note-off
    else
      it->second.disconnect();

    return true;
  }

  switch (event->keyval) {
  case GDK_Up:       state_manager->KeyPress(KeyUp);      break;
  case GDK_Down:     state_manager->KeyPress(KeyDown);    break;
  case GDK_Left:     state_manager->KeyPress(KeyLeft);    break;
  case GDK_Right:    state_manager->KeyPress(KeyRight);   break;
  case GDK_space:    state_manager->KeyPress(KeySpace);   break;
  case GDK_Return:   state_manager->KeyPress(KeyEnter);   break;
  case GDK_Escape:   state_manager->KeyPress(KeyEscape);  break;

  // show FPS
  case GDK_F6:       state_manager->KeyPress(KeyF6);      break;

  // increase/decrease octave
  case GDK_greater:  state_manager->KeyPress(KeyGreater); break;
  case GDK_less:     state_manager->KeyPress(KeyLess);    break;

  // +/- 5 seconds
  case GDK_Page_Down:state_manager->KeyPress(KeyForward);  break;
  case GDK_Page_Up:  state_manager->KeyPress(KeyBackward); break;

  case GDK_bracketleft:  state_manager->KeyPress(KeyVolumeDown); break; // [
  case GDK_bracketright: state_manager->KeyPress(KeyVolumeUp);   break; // ]

  default:
    return false;
  }

  return true;
}

bool DrawingArea::on_key_release(GdkEventKey* event) {

  // if is a note...
  int note = keyToNote(event);
  if (note >= 0) {

    // setup a timeout with Note-Off
    pressed[note] = Glib::signal_timeout().connect(
        sigc::bind<int>(sigc::ptr_fun(&__sendNoteOff), note), 20);
    return true;
  }

  return false;
}

void DrawingArea::on_realize() {
  // we need to call the base on_realize()
  Gtk::GL::DrawingArea::on_realize();

  Glib::RefPtr<Gdk::GL::Window> glwindow = get_gl_window();
  if (!glwindow->gl_begin(get_gl_context()))
    return;

  glwindow->gl_end();
}

bool DrawingArea::on_configure_event(GdkEventConfigure* event) {

  Glib::RefPtr<Gdk::GL::Window> glwindow = get_gl_window();
  if (!glwindow->gl_begin(get_gl_context()))
    return false;

  glClearColor(.25, .25, .25, 1.0);
  glClearDepth(1.0);

  glDisable(GL_DEPTH_TEST);
  glEnable(GL_TEXTURE_2D);

  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
  glEnable(GL_BLEND);

  glShadeModel(GL_SMOOTH);

  glViewport(0, 0, get_width(), get_height());
  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  gluOrtho2D(0, get_width(), 0, get_height());

  state_manager->Update(window_state.JustActivated());

  glwindow->gl_end();
  return true;
}

bool DrawingArea::on_expose_event(GdkEventExpose* event) {

  Glib::RefPtr<Gdk::GL::Window> glwindow = get_gl_window();
  if (!glwindow->gl_begin(get_gl_context()))
    return false;

  glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
  glCallList(1);

  Renderer rend(get_gl_context(), get_pango_context());
  rend.SetVSyncInterval(1);
  state_manager->Draw(rend);

  // swap buffers.
  if (glwindow->is_double_buffered())
     glwindow->swap_buffers();
  else
     glFlush();

  glwindow->gl_end();
  return true;
}

bool DrawingArea::GameLoop() {

  if (window_state.IsActive()) {

    state_manager->Update(window_state.JustActivated());

    Renderer rend(get_gl_context(), get_pango_context());
    rend.SetVSyncInterval(1);

    state_manager->Draw(rend);
  }

  return true;
}

int main(int argc, char *argv[]) {
  Gtk::Main main_loop(argc, argv);
  Gtk::GL::init(argc, argv);

  state_manager = new GameStateManager(
		  	  	  	  	  Compatible::GetDisplayWidth(),
						  Compatible::GetDisplayHeight()
					);

  try {
    string command_line("");

    UserSetting::Initialize(application_name);

    if (argc > 1)
      command_line = string(argv[1]);

    // strip any leading or trailing quotes from the filename
    // argument (to match the format returned by the open-file
    // dialog later).
    if (command_line.length() > 0 &&
	command_line[0] == '\"')
      command_line = command_line.substr(1, command_line.length() - 1);

    if (command_line.length() > 0 &&
	command_line[command_line.length()-1] == '\"')
      command_line = command_line.substr(0, command_line.length() - 1);

    Midi *midi = 0;

    // attempt to open the midi file given on the command line first
    if (command_line != "") {
      try {
	midi = new Midi(Midi::ReadFromFile(command_line));
      }

      catch (const MidiError &e) {
	string wrapped_description = STRING("Problem while loading file: " <<
					    command_line <<
					    "\n") + e.GetErrorDescription();
	Compatible::ShowError(wrapped_description);

	command_line = "";
	midi = 0;
      }
    }

    // if midi couldn't be opened from command line filename or there
    // simply was no command line filename, use a "file open" dialog.
    if (command_line == "") {
      while (!midi) {
	string file_title;
	FileSelector::RequestMidiFilename(&command_line, &file_title);

	if (command_line != "") {
	  try {
	    midi = new Midi(Midi::ReadFromFile(command_line));
	  }
	  catch (const MidiError &e) {
	    string wrapped_description = \
	      STRING("Problem while loading file: " <<
		     file_title <<
		     "\n") + e.GetErrorDescription();
	    Compatible::ShowError(wrapped_description);

	    midi = 0;
	  }
	}

	else {
	  // they pressed cancel, so they must not want to run
	  // the app anymore.
	  return 0;
	}
      }
    }

    Glib::RefPtr<Gdk::GL::Config> glconfig;

    // try double-buffered visual
    glconfig = Gdk::GL::Config::create(Gdk::GL::MODE_RGB    |
        			       Gdk::GL::MODE_DEPTH  |
        			       Gdk::GL::MODE_DOUBLE);
    if (!glconfig) {
      cerr << "*** Cannot find the double-buffered visual.\n"
           << "*** Trying single-buffered visual.\n";

      // try single-buffered visual
      glconfig = Gdk::GL::Config::create(Gdk::GL::MODE_RGB |
                                         Gdk::GL::MODE_DEPTH);
      if (!glconfig) {
	string description = STRING(error_header1 <<
				    " OpenGL" <<
				    error_header2 <<
				    "Cannot find any OpenGL-capable visual." <<
				    error_footer);
	Compatible::ShowError(description);
	return 1;
      }
    }

    Gtk::Window window;
    DrawingArea da(glconfig);
    window.add(da);
    window.show_all();
    window.move(Compatible::GetDisplayLeft() + Compatible::GetDisplayWidth()/2, Compatible::GetDisplayTop() + Compatible::GetDisplayHeight()/2);


    // Init DHMS thread once for the whole program
    DpmsThread* dpms_thread = new DpmsThread();

    // do this after gl context is created (ie. after da realized)
    SharedState state;
    state.song_title = FileSelector::TrimFilename(command_line);
    state.midi = midi;
    state.dpms_thread = dpms_thread;
    state_manager->SetInitialState(new TitleState(state));

    window.fullscreen();
    window.set_title(friendly_app_name);

    window.set_icon_from_file(string(GRAPHDIR) + "/app_icon.ico");

    // get refresh rate from user settings
    string key = "refresh_rate";
    int rate = 65;
    string user_rate = UserSetting::Get(key, "");
    if (user_rate.empty()) {
      user_rate = STRING(rate);
      UserSetting::Set(key, user_rate);
    }

    else {
      istringstream iss(user_rate);
      if (not (iss >> rate)) {
        Compatible::ShowError("Invalid setting for '"+ key +"' key.\n\nReset to default value when reload.");
        UserSetting::Set(key, "");
      }
    }

    Glib::signal_timeout().connect(sigc::mem_fun(da, &DrawingArea::GameLoop), 1000/rate);

    main_loop.run(window);
    window_state.Deactivate();

    delete dpms_thread;

    return 0;
  }

  catch (const LinthesiaError &e) {
    string wrapped_description = STRING(error_header1 <<
					error_header2 <<
					e.GetErrorDescription() <<
					error_footer);
    Compatible::ShowError(wrapped_description);
  }

  catch (const MidiError &e) {
    string wrapped_description = STRING(error_header1 <<
					" MIDI" <<
					error_header2 <<
					e.GetErrorDescription() <<
					error_footer);
    Compatible::ShowError(wrapped_description);
  }

  catch (const Gnome::Conf::Error& e) {
    string wrapped_description = STRING(error_header1 <<
					" Gnome::Conf::Error" <<
					error_header2 <<
					e.what() <<
					error_footer);
    Compatible::ShowError(wrapped_description);
  }

  catch (const exception &e) {
    string wrapped_description = STRING("Linthesia detected an unknown "
					"problem and must close!  '" <<
					e.what() << "'" << error_footer);
    Compatible::ShowError(wrapped_description);
  }

  catch (...) {
    string wrapped_description = STRING("Linthesia detected an unknown "
					"problem and must close!" <<
					error_footer);
    Compatible::ShowError(wrapped_description);
  }

  return 1;
}

