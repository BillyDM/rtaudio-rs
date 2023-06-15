use std::ffi::CStr;

use crate::NativeFormats;

/// A unique identifier for a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceID(pub u32);

/// Queried information about a device.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    /// The unique identifier of this device.
    pub id: DeviceID,
    /// The number of output channels on this device.
    pub output_channels: u32,
    /// The number of input channels on this device.
    pub input_channels: u32,
    /// The number of duplex channels on this device.
    pub duplex_channels: u32,

    /// Whether or not this device is the default output device.
    pub is_default_output: bool,
    /// Whether or not this device is the default input device.
    pub is_default_input: bool,

    /// The native sample formats that this device supports. (bitflags)
    ///
    /// Note you can still start a stream with any format. RtAudio will
    /// just automatically convert to/from the best native format.
    pub native_formats: NativeFormats,

    /// The device's preferred sample rate.
    pub preferred_sample_rate: u32,
    /// The available sample rates for this device.
    pub sample_rates: Vec<u32>,

    /// The display name of the device.
    pub name: String,
}

impl DeviceInfo {
    pub fn from_raw(d: rtaudio_sys::rtaudio_device_info_t) -> Self {
        let mut sample_rates = Vec::new();
        for sr in d.sample_rates.iter() {
            if *sr <= 0 {
                break;
            }

            sample_rates.push(*sr as u32);
        }

        // Safe because i8 and u8 have the same size, and we are correctly
        // using the length of the array `d.name`.
        let name_slice: &[u8] =
            unsafe { std::slice::from_raw_parts(d.name.as_ptr() as *const u8, d.name.len()) };

        let name = match CStr::from_bytes_until_nul(&name_slice) {
            Ok(n) => n.to_string_lossy().to_string(),
            Err(e) => {
                log::error!("RtAudio: Failed to parse audio device name: {}", e);

                String::from("error")
            }
        };

        Self {
            id: DeviceID(d.id as u32),
            output_channels: d.output_channels as u32,
            input_channels: d.input_channels as u32,
            duplex_channels: d.duplex_channels as u32,
            is_default_output: d.is_default_output != 0,
            is_default_input: d.is_default_input != 0,
            native_formats: NativeFormats::from_bits_truncate(d.native_formats),
            preferred_sample_rate: d.preferred_sample_rate as u32,
            sample_rates,
            name,
        }
    }
}
