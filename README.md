# linthesia

[![Build Status](https://travis-ci.org/linthesia/linthesia.svg?branch=master)](https://travis-ci.org/linthesia/linthesia)
[![Join the chat at https://gitter.im/linthesia/linthesia](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/linthesia/linthesia?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Linthesia is a fork of the Windows/Mac game called Synthesia. It is a game of playing music using a MIDI keyboard (or your PC keyboard), following a .mid file.

Synthesia up to version 0.6.1a is Open Source. This project uses the latest source from sourceforge.

## Compile

To compile, you need a basic c++ toolchain, and satisfy all dependences which are on BUILD-DEPENDS file. Then, just:

    $ ./autogen.sh

Here you must choose:

 a) For developers

    $ mkdir build
    $ cd build     # Isolate compilation to speed future compilations
    $ ../configure

 b) For general public

    $ ../configure --prefix=/usr

Then:

    $ make
    $ sudo make install

## Credits

Visit https://github.com/linthesia/linthesia for more info.
