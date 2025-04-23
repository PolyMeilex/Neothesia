# Video encoding

### For Linux and Windows

You can download `neothesia-cli` from [releasses](https://github.com/PolyMeilex/Neothesia/releases)

### For macOS

To encode video you need to install `rust` and `ffmpeg`.

Then compile the neothesia-cli: `make build-recorder`

It will compile neothesia-cli, from now on it is used as a command line tool

To encode a test.mid file run `./target/release/neothesia-cli ./test.mid`

Video will be outputted to `./out` directory
