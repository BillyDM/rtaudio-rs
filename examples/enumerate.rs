fn main() {
    dbg!(rtaudio::version());

    for api in rtaudio::compiled_apis() {
        dbg!(api.get_display_name());

        match rtaudio::Host::new(api) {
            Ok(rt) => {
                for device_info in rt.iter_devices() {
                    dbg!(device_info);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        println!("---------------------------------------------");
    }
}
