use rtaudio_sys::MAX_NAME_LENGTH;
use std::ffi::CString;
use std::os::raw::c_char;

use crate::error::{RtAudioError, RtAudioErrorType};
use crate::StreamFlags;

/// Used for specifying the parameters of a device when opening a
/// stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceParams {
    /// The ID (not index) of the device to use.
    pub device_id: u32,
    /// The number of channels in the device to use.
    pub num_channels: u32,
    /// The first channel index on the device (default = 0) to use.
    pub first_channel: u32,
}

impl Default for DeviceParams {
    fn default() -> Self {
        Self {
            device_id: 0,
            num_channels: 2,
            first_channel: 0,
        }
    }
}

impl DeviceParams {
    pub fn to_raw(&self) -> rtaudio_sys::rtaudio_stream_parameters_t {
        rtaudio_sys::rtaudio_stream_parameters_t {
            device_id: self.device_id,
            num_channels: self.num_channels,
            first_channel: self.first_channel,
        }
    }
}

/// Additional options for opening a stream.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamOptions {
    /// The bit flag parameters for this stream.
    ///
    /// By default, no flags are set.
    pub flags: StreamFlags,

    /// Used to control stream latency in the Windows DirectSound, Linux OSS, and Linux Alsa APIs only.
    /// A value of two is usually the smallest allowed. Larger numbers can potentially result in more
    /// robust stream performance, though likely at the cost of stream latency.
    ///
    /// The actual value used when the stream is ran may be different.
    ///
    /// The default value is `4`.
    pub num_buffers: u32,

    /// Scheduling priority of callback thread (only used with flag `StreamFlags::SCHEDULE_REALTIME`).
    ///
    /// Use a value of `-1` for the default priority.
    ///
    /// The default value is `-1`.
    pub priority: i32,

    /// The name of the stream (currently used only in Jack).
    ///
    /// The size of the name cannot exceed 511 bytes.
    pub name: String,
}

impl StreamOptions {
    pub fn to_raw(&self) -> Result<rtaudio_sys::rtaudio_stream_options_t, RtAudioError> {
        let name = str_to_c_array::<{ MAX_NAME_LENGTH as usize }>(&self.name).map_err(|_| {
            RtAudioError {
                type_: RtAudioErrorType::InvalidParamter,
                msg: Some("stream name is invalid".into()),
            }
        })?;

        Ok(rtaudio_sys::rtaudio_stream_options_t {
            flags: self.flags.bits(),
            num_buffers: self.num_buffers,
            priority: self.priority,
            name,
        })
    }
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            flags: StreamFlags::empty(),
            num_buffers: 4,
            priority: -1,
            name: String::from("RtAudio-rs Client"),
        }
    }
}

fn str_to_c_array<const MAX_LEN: usize>(s: &str) -> Result<[c_char; MAX_LEN], ()> {
    let cs = CString::new(s).map_err(|_| ())?;
    let cs_slice = cs.as_bytes_with_nul();

    // Safe because i8 and u8 have the same size.
    let cs_slice =
        unsafe { std::slice::from_raw_parts(cs_slice.as_ptr() as *const c_char, cs_slice.len()) };

    if cs_slice.len() > MAX_LEN as usize {
        return Err(());
    }

    let mut c_array: [c_char; MAX_LEN] = [0; MAX_LEN];

    c_array[0..cs_slice.len()].copy_from_slice(&cs_slice);

    Ok(c_array)
}
