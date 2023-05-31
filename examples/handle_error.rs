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
