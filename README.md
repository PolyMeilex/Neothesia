# Full Rewrite Of Neothesia In Rust

![Neothesia Baner](https://i.imgur.com/QfdMwMI.png)

## This version of Neothesia is not functional yet, Use: [c++ version](https://github.com/PolyMeilex/Neothesia/tree/master)

So I decided to drop old c++ codebase of synthesia in favour of my own (hopefully) cleaner implementation.

Cross platform support in current version of Neothesia is nearly non existent because of the way original synthesia was written, and instead of trying to patch cross platform support with hack'y code, I decided to rewrite it from ground up with cross platform in mind.
It is also occasion to rewrite whole Neothesia to Rust (language that I'am more familiar with than c++)

## First Working Prototype

[![Video](https://i.imgur.com/t0IaVA1.png)](https://youtu.be/1fsii7kQDw0)
[Video](https://youtu.be/1fsii7kQDw0)

## TODO:

- [x] Midi File
  - [x] Read File
  - [x] Calculate Note Start Times And Duration
  - [x] Temporary Midi Player
  - [ ] Proper Midi Player On Separate Thread
  - [x] Fix Desynchronization Of Tracks When Tempo Is Changing In The Middle Of Song (/tests/ExampleOfBrokenTempo.mid)
- [x] Midi Connection
  - [ ] Create Midi Connection Wrapper
- [x] Rendering
  - [ ] Cross Platform File Select Dialogue
  - [x] Notes Waterfall Shader
  - [x] Keyboard Renderer
  - [ ] Particles Shader
  - [ ] Particles BG Blur Shader
  - [ ] Game Main Menu (a beautiful one :smiley: )
