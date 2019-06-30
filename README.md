# Full Rewrite Of Nothesia In Rust
So I decided to drop old c++ codebase of synthesia in favour of my own (hopefully) cleaner implementation.

Cross platform support in current version of Nothesia is nearly non existent because of the way original synthesia was written, and instead of trying to path cross platform support with hack'y code, I decided to rewrite it from ground up with cross platform in mind.
It is also occasion to rewrite whole Nothesia to Rust (language that I'am more familiar with than c++)

## TODO:
* [x] Midi File
    * [x] Read File
    * [x] Calculate Note Start Times And Duration
    * [x] Temporary Midi Player
    * [ ] Proper Midi Player
    * [ ] Fix Desynchronization Of Tracks When Tempo Is Changing In The Middle Of Song
* [ ] Midi Connection
    * [ ] Create Midi Connection Wrapper
* [ ] Rendering
    * [ ] SDL2 (or some alternative lib)
    * [ ] Cross Platform File Select Dialogue
    * [ ] FBO Wrapper
    * [ ] Shader Loading System
    * [ ] Notes Waterfall Shader
    * [ ] Keyboard Renderer
    * [ ] Particles Shader
    * [ ] Particles BG Blur Shader
    * [ ] Game Main Menu (a beautiful one :smiley: )