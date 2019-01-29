![Neothesia Baner](https://i.imgur.com/3uiwId8.png)
# Neothesia
Neothesia is a fork of [Legacy Synthesia](https://github.com/johndpope/pianogame)
Opensource Synthesia was abandoned in favor of [closed source commercial project](https://www.synthesiagame.com/)
Goal of this project is to bring back Legacy Synthesia to live, and make it look as good (or even better) than commercial Synthesia.

Linux build is based on [Linthesia](https://github.com/linthesia/linthesia)

## Goals
* Make it look like modern software
* Treat Linux users as first class citizens
* Give Linux users good or even better alternative to windows Synthesia
* Make it flashy, particles and other cool effects
* Make it as friendly as possible for youtube piano tutorials creators (like myself)
* (Maybe) Support Windows in future

## Windows Build?
For now I will not support windows build 😰, it is really annoying to open VM every time I want to test something, if you are windows user and you have bare minimum c++ knowledge, you can easly port it yourself.   
I would realy appreciate any help in maintaining windows branch. 

If you want to become full time windows maintainer you are welcome to do so,  
You will have special place in my heart if you do so 😉

##### Noob Friendly Way To Port (not recomended)
* clone [Legacy Synthesia](https://github.com/johndpope/pianogame)
* add Neothesia commits
* build


## Todo
* Make it cross platform so ports are no longer needed
* Replace unnecessary sprites with shaders
* Create proper shader loading system instead of current placeholder one
* Modernise main screen and track selection screen
* Consider adding song select screen, instead of native dialog

## Compile

To compile, you need a basic c++ toolchain, and satisfy all dependences which are on BUILD-DEPENDS file. Then, just:

    $ autoreconf -ivf
    $ mkdir build
    $ cd build     # Isolate compilation to speed future compilations
    
Here you must choose:

 a) For developers
 
    $ ../configure
 b) For general public

    $ ../configure --prefix=/usr

Then: (still in build directory)

    $ make
    $ sudo make install

## Credits
* Linux Build is based on master branch of [Linthesia](https://github.com/linthesia/linthesia)
* Windows Build is based on master branch of [Legacy Synthesia](https://github.com/johndpope/pianogame)

