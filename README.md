# RtAudio-rs
[![Documentation](https://docs.rs/rtaudio/badge.svg)](https://docs.rs/rtaudio)
[![Crates.io](https://img.shields.io/crates/v/rtaudio.svg)](https://crates.io/crates/rtaudio)
[![License](https://img.shields.io/crates/l/rtaudio.svg)](https://github.com/BillyDM/rtaudio-rs/blob/main/LICENSE)

Safe Rust wrapper and bindings for [RtAudio](https://github.com/thestk/rtaudio).

# Usage Example

```rust
use rtaudio::{Api, Buffers, DeviceParams, SampleFormat, StreamInfo, StreamOptions, StreamStatus};

fn main() {
    let host = rtaudio::Host::new(Api::Unspecified).unwrap();
    let out_device = host.default_output_device().unwrap();

    let (error_tx, error_rx) = std::sync::mpsc::channel();

    let mut stream = host
        .open_stream(
            Some(DeviceParams {
                device_id: out_device.id,
                num_channels: 2,
                first_channel: 0,
            }),
            None,
            SampleFormat::Float32,
            out_device.preferred_sample_rate,
            256,
            StreamOptions::default(),
            move |error| error_tx.send(error).unwrap(),
        )
        .unwrap();

    let mut phasor = 0.0;
    let phasor_inc = 440.0 / stream.info().sample_rate as f32;

    stream
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input: _ } = buffers {
                    let frames = output.len() / 2;

                    for i in 0..frames {
                        let val = (phasor * std::f32::consts::TAU).sin() * 0.5;

                        // By default, buffers are interleaved.
                        output[i * 2] = val;
                        output[(i * 2) + 1] = val;

                        phasor = (phasor + phasor_inc).fract();
                    }
                }
            },
        )
        .unwrap();

    if let Ok(error) = error_rx.recv_timeout(std::time::Duration::from_millis(5000)) {
        // An error occured that caused the stream to close (for example a
        // device was unplugged). Now our stream object should be manually
        // closed or dropped.
        eprintln!("{}", error);
    }
}
```

# Prerequisites

`CMake` is required on all platforms.

## Linux

```
apt install cmake pkg-config libasound2-dev libpulse-dev
```

If the `jack_linux` feature is enabled, then also install the jack development headers:
```
apt install libjack-dev
```

## MacOS

### Install CMake: Option 1

Download at https://cmake.org/.

### Install CMake: Option 2

Install with [Homebrew](https://brew.sh/):

```
brew install cmake
```

## Windows

### Install CMake

Download at https://cmake.org/.

# Features

By default, Jack on Linux and ASIO on Windows is disabled. You can enable them with the `jack_linux` and `asio` features.

```
rtaudio-sys = { version = "0.1", features = ["jack_linux", "asio"] }
```

# Notes

Bindings were made from the official [C header](https://github.com/thestk/rtaudio/blob/master/rtaudio_c.h). No bindings to the C++ interface are provided.

This currently builds a static library from source on all platforms. Once RtAudio version 6 stabilizes I might have it link to the dynamic RtAudio library on Linux.

I haven't figured out how to get Jack on MacOS to work yet. If you know how to install and link the Jack libraries on MacOS, please let me know.

I haven't thoroughly tested every API on every platform yet. If you run into any bugs or issues with building, please create an issue.