# videotaxi-speech-rs

This is a Rust library for interacting with the VIDEO.TAXI Speech API.

An example implementation can be found in the `src/example` directory. You can run this example easily with the following command:

```bash
ffmpeg -f alsa -i default -ac 2 -f adts -  | VIDEOTAXI_TOKEN=INSERT_ME cargo run --example basic
```

## Requirements

- tested with rustc 1.88
- a video taxi token
- ffmpeg for running the example


## Installation

```bash
cargo install --git https://github.com/TV1-EU/videotaxi-speech-rs
```


## Questions?

Just open an issue on GitHub or contact us via the support channels https://support.video.taxi
