// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __FILE_SELECTOR_H
#define __FILE_SELECTOR_H

#include <string>

namespace FileSelector {

  // Presents a standard "File Open" dialog box. Returns empty string 
  // in [filename] if user presses cancel.  Also, remembers last filename
  void RequestMidiFilename(std::string *filename, std::string *file_title);

  // If a filename was passed in on the command line, we
  // can remember it for future file-open dialogs
  void SetLastMidiFilename(const std::string &filename);
  
  // Returns a filename with no path or .mid/.midi extension
  std::string TrimFilename(const std::string &filename);
};

#endif  // __FILE_SELECTOR_H

