use rtaudio::{Api, Buffers, DeviceParams, SampleFormat, StreamInfo, StreamOptions, StreamStatus};

const AMPLITUDE: f32 = 0.5;
const FREQ_HZ: f32 = 440.0;

fn main() {
    let host = rtaudio::Host::new(Api::Unspecified).unwrap();
    dbg!(host.api());

    let out_device = host.default_output_device().unwrap();

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
            |error| eprintln!("{}", error),
        )
        .unwrap();
    dbg!(stream.info());

    let mut phasor = 0.0;
    let phasor_inc = FREQ_HZ / stream.info().sample_rate as f32;

    stream
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input: _ } = buffers {
                    let frames = output.len() / 2;

                    for i in 0..frames {
                        let val = (phasor * std::f32::consts::TAU).sin() * AMPLITUDE;

                        // By default, buffers are interleaved.
                        output[i * 2] = val;
                        output[(i * 2) + 1] = val;

                        phasor = (phasor + phasor_inc).fract();
                    }
                }
            },
        )
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(3000));
}
