//! Demonstrates how to handle stream errors.

use rtaudio::{Api, Buffers, DeviceParams, SampleFormat, StreamInfo, StreamOptions, StreamStatus};
use std::time::{Duration, Instant};

fn main() {
    let host = rtaudio::Host::new(Api::Unspecified).unwrap();
    let out_device = host.default_output_device().unwrap();

    let (error_tx, error_rx) = std::sync::mpsc::channel();

    let mut stream_handle = host
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

    stream_handle
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input: _ } = buffers {
                    // Fill the output with silence.
                    output.fill(0.0);
                }
            },
        )
        .unwrap();

    // Play for 5 seconds and then close.
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(5) {
        // Periodically poll to see if an error has happened.
        if let Ok(error) = error_rx.try_recv() {
            // An error occured that caused the stream to close (for example a
            // device was unplugged). Now our stream_handle object should be
            // manually closed or dropped.
            eprintln!("{}", error);

            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}
