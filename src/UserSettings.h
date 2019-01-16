// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __USER_SETTINGS_H
#define __USER_SETTINGS_H

#include <string>

namespace UserSetting {

   // This must be called exactly once before any of the following will work
   void Initialize(const std::string &app_name);

   std::string Get(const std::string &setting,
		   const std::string &default_value);
  
   void Set(const std::string &setting, 
	    const std::string &value);
};

#endif // __USER_SETTINGS_H
