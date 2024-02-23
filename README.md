![Neothesia Baner](https://github.com/PolyMeilex/Neothesia/assets/20758186/ca9aa8ae-2a69-48de-92d6-97d7ea9e678d)

# Neothesia

Opensource Synthesia was abandoned in favour of [closed source commercial project](https://www.synthesiagame.com/)  
Goal of this project is to bring back Opensource Synthesia to live, and make it look and work as good (or even better) than commercial Synthesia.

If you have any questions, feel free to join my Discord

[<img alt="Discord" src="https://img.shields.io/discord/273176778946641920?logo=discord&style=for-the-badge&color=%23a051ee">](https://discord.gg/sgeZuVA)

## Screenshots

![image](https://github.com/PolyMeilex/Neothesia/assets/20758186/65483bab-0b74-4fd4-90b1-fdd00508b676)

[![Video](https://github.com/PolyMeilex/Neothesia/assets/20758186/dc564433-aade-4430-b137-5f90000ae9e0)](https://youtu.be/ReE9nVuMCSE)

|![settings](https://github.com/PolyMeilex/Neothesia/assets/20758186/e38642e2-6118-4931-9964-a1df27a36db9)|![track selection](https://github.com/PolyMeilex/Neothesia/assets/20758186/2309d970-0234-45ff-a9f4-105ff08514af)|
|--|--|

[Video](https://youtu.be/ReE9nVuMCSE)

## Download

<a href="https://flathub.org/apps/details/com.github.polymeilex.neothesia"><img width="240" alt="Download on Flathub" src="https://flathub.org/assets/badges/flathub-badge-en.png"/></a>

Arch Linux (**Unofficial AUR** built from source, maintained by @zayn7lie): <https://aur.archlinux.org/packages/neothesia>

All binary releases:
[https://github.com/PolyMeilex/Neothesia/releases](https://github.com/PolyMeilex/Neothesia/releases)

## FAQ

- [FAQ](https://github.com/PolyMeilex/Neothesia/wiki/FAQ)

## Video encoding

- For Linux and Windows you can download neothesia-cli / recorder build from releasses
- For macOS
    - To encode video you need to install [rust](https://www.rust-lang.org/)
    - You also need to install [ffmpeg](https://ffmpeg.org/)
    - And compile the `neothesia-cli`, like so `cargo build --release -p neothesia-cli` (if you have make: `make build-recorder`)
    - It will compile `neothesia-cli`, from now on it is used as a cmomand line tool
- To encode a `test.mid` file run `./target/release/neothesia-cli ./test.mid`
- Video will be outputted to `./out` directory`

## Thanks to

- [WGPU](https://wgpu.rs/)
- [Linthesia](https://github.com/linthesia/linthesia)
- [Synthesia](https://github.com/johndpope/pianogame)
