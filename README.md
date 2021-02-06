![Neothesia Baner](https://i.imgur.com/QfdMwMI.png)

# Neothesia
Opensource Synthesia was abandoned in favour of [closed source commercial project](https://www.synthesiagame.com/)  
Goal of this project is to bring back Opensource Synthesia to live, and make it look and work as good (or even better) than commercial Synthesia.

If you have any questions, feel free to join my Discord

[<img alt="Discord" src="https://img.shields.io/discord/273176778946641920?logo=discord&style=for-the-badge&color=%23a051ee">](https://discord.gg/sgeZuVA)
## Screenshots
[![IMG](https://i.imgur.com/WUO61EN.png)](https://youtu.be/ReE9nVuMCSE)
[Video](https://youtu.be/ReE9nVuMCSE)
[![Video](https://i.imgur.com/1R5uOnA.png)](https://youtu.be/ReE9nVuMCSE)
## Download
[https://github.com/PolyMeilex/Neothesia/releases](https://github.com/PolyMeilex/Neothesia/releases)
## FAQ
- [FAQ](https://github.com/PolyMeilex/Neothesia/wiki/FAQ)


## Video encoding
- To encode video you need to install [rust](https://www.rust-lang.org/)
- You also need to install [ffmpeg](https://ffmpeg.org/)
- And compile the project with `record` flag, like so `cargo build --release --no-default-features --features record` (if you have make: `make build-recorder`)
- It will compile `neothesia` in recorder mode, from now on it is used as a comand line tool
- To encode a `test.mid` file run `./target/release/neothesia ./test.mid`
- Video will be outputed to `./out` directory`

## Thanks to
- [WGPU](https://wgpu.rs/)
- [Linthesia](https://github.com/linthesia/linthesia)
- [Synthesia](https://github.com/johndpope/pianogame)
