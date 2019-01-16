// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <gconfmm.h>

#include "StringUtil.h"

using namespace std;

namespace UserSetting {

  static bool g_initialized(false);
  static string g_app_name("");
  static Glib::RefPtr<Gnome::Conf::Client> gconf;

  void Initialize(const string &app_name) {
    if (g_initialized) 
      return;

    Gnome::Conf::init(); 

    gconf = Gnome::Conf::Client::get_default_client();
    g_app_name = "/apps/" + app_name;
    g_initialized = true;
  }

  string Get(const string &setting, const string &default_value) {
    if (!g_initialized) 
      return default_value;

    string result = gconf->get_string(g_app_name + "/" + setting);
    if (result.empty())
      return default_value;
    
    return result;
  }
    
  void Set(const string &setting, const string &value) {
    if (!g_initialized) 
      return;

    gconf->set(g_app_name + "/" + setting, value);
  }

}; // End namespace
