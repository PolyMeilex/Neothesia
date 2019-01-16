// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <gtkmm.h>
#include <set>

#include "LinthesiaError.h"
#include "FileSelector.h"
#include "UserSettings.h"
#include "StringUtil.h"

using namespace std;

const static char PathDelimiter = '/';

namespace FileSelector {

  void RequestMidiFilename(string *returned_filename,
			   string *returned_file_title) {

    // Grab the filename of the last song we played
    // and pre-load it into the open dialog
    string last_filename = UserSetting::Get("last_file", "");

    Gtk::FileChooserDialog dialog("Linthesia: Choose a MIDI song to play");
    dialog.add_button(Gtk::StockID("gtk-open"), Gtk::RESPONSE_ACCEPT);
    dialog.add_button(Gtk::StockID("gtk-cancel"), Gtk::RESPONSE_CANCEL);

    // Try to populate our "File Open" box with the last file selected
    if (!last_filename.empty())
      dialog.set_filename(last_filename);

    // If there wasn't a last file, default to the built-in Music directory
    else {
      string default_dir = UserSetting::Get("default_music_directory", "");
      dialog.set_current_folder(default_dir);
    }

    // Set file filters
    Gtk::FileFilter filter_midi;
    filter_midi.set_name("MIDI files (*.mid, *.midi)");
    filter_midi.add_pattern("*.mid");
    filter_midi.add_pattern("*.midi");
    dialog.add_filter(filter_midi);

    Gtk::FileFilter filter_all;
    filter_all.set_name("All files (*.*)");
    filter_all.add_pattern("*.*");
    dialog.add_filter(filter_all);

    int response = dialog.run();
    switch (response) {
    case Gtk::RESPONSE_ACCEPT:

      string filename = dialog.get_filename();
      SetLastMidiFilename(filename);

      if (returned_file_title)
	*returned_file_title = filename.substr(filename.rfind(PathDelimiter)+1);

      if (returned_filename)
	*returned_filename = filename;

      return;
    }

    if (returned_file_title)
      *returned_file_title = "";

    if (returned_filename)
      *returned_filename = "";
  }

  void SetLastMidiFilename(const string &filename) {
    UserSetting::Set("last_file", filename);
  }

  string TrimFilename(const string &filename) {

    // lowercase
    string lower = StringLower(filename);

    // remove extension, if known
    set<string> exts;
    exts.insert(".mid");
    exts.insert(".midi");
    for (set<string>::const_iterator i = exts.begin(); i != exts.end(); i++) {
      int len = i->length();
      if (lower.substr(lower.length() - len, len) == *i)
        lower = lower.substr(0, lower.length() - len);
    }

    // remove path
    string::size_type i = lower.find_last_of("/");
    if (i != string::npos)
      lower = lower.substr(i+1, lower.length());

    return lower;
  }

}; // End namespace
