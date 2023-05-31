fn main() {
    dbg!(rtaudio::version());

    let compiled_apis = rtaudio::compiled_apis();
    for api in compiled_apis {
        dbg!(api);
        dbg!(api.get_name());
        dbg!(api.get_display_name());

        match rtaudio::Host::new(api) {
            Ok(rt) => {
                dbg!(rt.api());

                for device_res in rt.iter_devices() {
                    match device_res {
                        Ok(device) => {
                            dbg!(device);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        println!("---------------------------------------------");
    }
}
