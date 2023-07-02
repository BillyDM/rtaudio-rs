use rtaudio::{Api, Buffers, DeviceParams, SampleFormat, StreamInfo, StreamOptions, StreamStatus};

fn main() {
    let host = rtaudio::Host::new(Api::Unspecified).unwrap();
    dbg!(host.api());

    let out_device = host.default_output_device().unwrap();
    let in_device = host.default_input_device().unwrap();

    let mut stream_handle = host
        .open_stream(
            Some(DeviceParams {
                device_id: out_device.id,
                num_channels: 2,
                first_channel: 0,
            }),
            Some(DeviceParams {
                device_id: in_device.id,
                num_channels: 2,
                first_channel: 0,
            }),
            SampleFormat::Float32,
            out_device.preferred_sample_rate,
            256,
            StreamOptions::default(),
            |error| eprintln!("{}", error),
        )
        .unwrap();
    dbg!(stream_handle.info());

    stream_handle
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input } = buffers {
                    // Copy the input to the output.
                    output.copy_from_slice(input);
                }
            },
        )
        .unwrap();

    // Wait 3 seconds before closing.
    std::thread::sleep(std::time::Duration::from_millis(3000));
}
