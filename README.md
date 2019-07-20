# Full Rewrite Of Neothesia In Rust

![Neothesia Baner](https://i.imgur.com/QfdMwMI.png)

## This version of Neothesia is not fully functional yet, maybe you should use: [c++ version](https://github.com/PolyMeilex/Neothesia/tree/master)

I decided to drop old c++ codebase of synthesia in favour of my own (hopefully) cleaner implementation.

Cross platform support in current version of Neothesia is nearly non existent because of the way original synthesia was written, and instead of trying to patch cross platform support with hack'y code, I decided to rewrite it from ground up with cross platform in mind.
It is also occasion to rewrite whole Neothesia to Rust (language that I'am more familiar with than c++)

## First Working Prototype
[![Video](https://i.imgur.com/t0IaVA1.png)](https://youtu.be/1fsii7kQDw0)
[Video](https://youtu.be/1fsii7kQDw0)
## Download
![IMG](https://i.snag.gy/F8SCbv.jpg)

First Test Builds Of Rust Neothesia
- [Linux x86_64](https://github.com/PolyMeilex/Neothesia/releases/download/v0.0.1-alpha/neothesia-linux-x86_64)
- Windows
  - [x86_64](https://github.com/PolyMeilex/Neothesia/releases/download/v0.0.1-alpha/neothesia-windows-x86_64.exe)
  - [i686](https://github.com/PolyMeilex/Neothesia/releases/download/v0.0.1-alpha/neothesia-windows-i686.exe)
- Totally Untested:
  - Mac - It is too annoying to compile for mac, if you want to do it yourself, I can help you with it on [Discord](https://discord.gg/fc9GZrc)

## TODO:

- [x] Midi File
  - [x] Read File
  - [x] Calculate Note Start Times And Duration
  - [x] Temporary Midi Player
  - [x] Proper Midi Player
    - [ ] Maybe Midi Player On Separate Thread For Black Midi :smiley:
  - [x] Fix Desynchronization Of Tracks When Tempo Is Changing In The Middle Of Song (/tests/ChangingTempo.mid)
- [x] Midi Connection
  - [x] Create Midi Connection Wrapper
- [x] Rendering
  - [x] Cross Platform File Select Dialogue
  - [x] Notes Waterfall Shader
  - [x] Keyboard Renderer
  - [ ] Add Better Controls For Navigating Tracks
  - [ ] Particles Shader
  - [ ] Particles BG Blur Shader
  - [x] Game Main Menu
