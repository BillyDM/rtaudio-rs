use bitflags::bitflags;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

bitflags! {
    /// The native formats this device supports.
    ///
    /// Support for signed integers and floats. Audio data fed to/from an RtAudio stream
    /// is assumed to ALWAYS be in host byte order. The internal routines will
    /// automatically take care of any necessary byte-swapping between the host format
    /// and the soundcard. Thus, endian-ness is not a concern in the following format
    /// definitions.
    ///
    /// Note you can still start a stream with any format. RtAudio will just
    /// automatically convert to/from the best native format.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct NativeFormats: rtaudio_sys::rtaudio_format_t {
        /// 8-bit signed integer.
        const SINT8 = rtaudio_sys::RTAUDIO_FORMAT_SINT8;
        /// 16-bit signed integer.
        const SINT16 = rtaudio_sys::RTAUDIO_FORMAT_SINT16;
        /// 24-bit signed integer.
        const SINT24 = rtaudio_sys::RTAUDIO_FORMAT_SINT24;
        /// 32-bit signed integer.
        const SINT32 = rtaudio_sys::RTAUDIO_FORMAT_SINT32;
        /// 32-bit floating point number, normalized between plus/minus 1.0.
        const FLOAT32 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT32;
        /// 64-bit floating point number, normalized between plus/minus 1.0.
        const FLOAT64 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT64;
    }
}

/// The sample format type.
///
/// Support for signed integers and floats. Audio data fed to/from an RtAudio stream
/// is assumed to ALWAYS be in host byte order. The internal routines will
/// automatically take care of any necessary byte-swapping between the host format
/// and the soundcard. Thus, endian-ness is not a concern in the following format
/// definitions.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SampleFormat {
    /// 8-bit signed integer.
    SInt8 = rtaudio_sys::RTAUDIO_FORMAT_SINT8 as usize,
    /// 16-bit signed integer.
    SInt16 = rtaudio_sys::RTAUDIO_FORMAT_SINT16 as usize,
    /// 24-bit signed integer.
    ///
    /// The endianness will always be in the host's native byte order.
    SInt24 = rtaudio_sys::RTAUDIO_FORMAT_SINT24 as usize,
    /// 32-bit signed integer.
    SInt32 = rtaudio_sys::RTAUDIO_FORMAT_SINT32 as usize,
    /// 32-bit floating point number, normalized between plus/minus 1.0.
    Float32 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT32 as usize,
    /// 64-bit floating point number, normalized between plus/minus 1.0.
    Float64 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT64 as usize,
}

impl SampleFormat {
    pub fn to_raw(&self) -> rtaudio_sys::rtaudio_format_t {
        match self {
            SampleFormat::SInt8 => rtaudio_sys::RTAUDIO_FORMAT_SINT8,
            SampleFormat::SInt16 => rtaudio_sys::RTAUDIO_FORMAT_SINT16,
            SampleFormat::SInt24 => rtaudio_sys::RTAUDIO_FORMAT_SINT24,
            SampleFormat::SInt32 => rtaudio_sys::RTAUDIO_FORMAT_SINT32,
            SampleFormat::Float32 => rtaudio_sys::RTAUDIO_FORMAT_FLOAT32,
            SampleFormat::Float64 => rtaudio_sys::RTAUDIO_FORMAT_FLOAT64,
        }
    }
}

impl Default for SampleFormat {
    fn default() -> Self {
        SampleFormat::Float32
    }
}

bitflags! {
    /// Stream option flags.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct StreamFlags: rtaudio_sys::rtaudio_stream_flags_t {
        /// Use non-interleaved buffers (default = interleaved).
        const NONINTERLEAVED = rtaudio_sys::RTAUDIO_FLAGS_NONINTERLEAVED;
        /// Attempt to set stream parameters for lowest possible latency, with the
        /// possible expense of stream performance.
        const MINIMIZE_LATENCY = rtaudio_sys::RTAUDIO_FLAGS_MINIMIZE_LATENCY;
        /// Attempt to grab the device for exclusive use.
        ///
        /// Note that this is not possible with all supported audio APIs.
        const HOG_DEVICE = rtaudio_sys::RTAUDIO_FLAGS_HOG_DEVICE;
        /// Attempt to select realtime scheduling (round-robin) for the callback thread.
        const SCHEDULE_REALTIME = rtaudio_sys::RTAUDIO_FLAGS_SCHEDULE_REALTIME;
        /// Attempt to open the "default" PCM device when using the ALSA API. Note that
        /// this will override any specified input or output device index.
        const ALSA_USE_DEFAULT = rtaudio_sys::RTAUDIO_FLAGS_ALSA_USE_DEFAULT;
        /// Do not automatically connect ports (JACK only).
        const JACK_DONT_CONNECT = rtaudio_sys::RTAUDIO_FLAGS_JACK_DONT_CONNECT;
    }
}

bitflags! {
    /// Stream status (over- or underflow) flags.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct StreamStatus: rtaudio_sys::rtaudio_stream_status_t {
        /// Input data was discarded because of an overflow condition at the driver.
        const INPUT_OVERFLOW = rtaudio_sys::RTAUDIO_STATUS_INPUT_OVERFLOW;
        /// The output buffer ran low, likely producing a break in the output sound.
        const OUTPUT_UNDERFLOW = rtaudio_sys::RTAUDIO_STATUS_OUTPUT_UNDERFLOW;
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Api {
    /// Search for a default working compiled API.
    Unspecified = rtaudio_sys::RTAUDIO_API_UNSPECIFIED as i32,
    /// Macintosh OS-X Core Audio API.
    MacOSXCore = rtaudio_sys::RTAUDIO_API_MACOSX_CORE as i32,
    /// The Advanced Linux Sound Architecture API.
    LinuxALSA = rtaudio_sys::RTAUDIO_API_LINUX_ALSA as i32,
    /// The Jack Low-Latency Audio Server API.
    UnixJack = rtaudio_sys::RTAUDIO_API_UNIX_JACK as i32,
    /// The Linux PulseAudio API.
    LinuxPulse = rtaudio_sys::RTAUDIO_API_LINUX_PULSE as i32,
    /// The Linux Open Sound System API.
    LinuxOSS = rtaudio_sys::RTAUDIO_API_LINUX_OSS as i32,
    /// The Steinberg Audio Stream I/O API.
    WindowsASIO = rtaudio_sys::RTAUDIO_API_WINDOWS_ASIO as i32,
    /// The Microsoft WASAPI API.
    WindowsWASAPI = rtaudio_sys::RTAUDIO_API_WINDOWS_WASAPI as i32,
    /// The Microsoft DirectSound API.
    WindowsDS = rtaudio_sys::RTAUDIO_API_WINDOWS_DS as i32,
    /// A compilable but non-functional API.
    Dummy = rtaudio_sys::RTAUDIO_API_DUMMY as i32,
}

impl Api {
    /// Get the short lower-case name used for identification purposes.
    ///
    /// This value is guaranteed to remain identical across library versions.
    ///
    /// If the API is unknown, this will return `None`.
    pub fn get_name(&self) -> String {
        let index = self.to_raw();

        // Safe because we assume that this function returns a valid C String,
        // we check for the null case, and we don't free the pointer.
        let s = unsafe {
            // For some odd reason, this is off by one.
            let raw_s = rtaudio_sys::rtaudio_api_name(index);
            if raw_s.is_null() {
                return String::from("error");
            }

            CStr::from_ptr(raw_s as *mut c_char)
                .to_string_lossy()
                .to_string()
        };

        s
    }

    /// Get the display name for the given API.
    ///
    /// If the API is unknown, this will return `None`.
    pub fn get_display_name(&self) -> String {
        let index = self.to_raw();

        // Safe because we assume that this function returns a valid C String,
        // we check for the null case, and we don't free the pointer.
        let s = unsafe {
            // For some odd reason, this is off by one.
            let raw_s = rtaudio_sys::rtaudio_api_display_name(index);
            if raw_s.is_null() {
                return String::from("error");
            }

            CStr::from_ptr(raw_s as *mut c_char)
                .to_string_lossy()
                .to_string()
        };

        s
    }

    /// Retrieve the API by its name (as given in Api::get_name()).
    pub fn from_name(name: &str) -> Option<Api> {
        let c_name = if let Ok(n) = CString::new(name) {
            n
        } else {
            return None;
        };

        // Safe because we have constructed a valid C String.
        let index = unsafe { rtaudio_sys::rtaudio_compiled_api_by_name(c_name.as_ptr()) };

        if let Some(a) = Self::from_raw(index) {
            if a == Api::Unspecified {
                None
            } else {
                Some(a)
            }
        } else {
            None
        }
    }

    pub fn from_raw(a: rtaudio_sys::rtaudio_api_t) -> Option<Api> {
        match a {
            rtaudio_sys::RTAUDIO_API_UNSPECIFIED => Some(Api::Unspecified),
            rtaudio_sys::RTAUDIO_API_MACOSX_CORE => Some(Api::MacOSXCore),
            rtaudio_sys::RTAUDIO_API_LINUX_ALSA => Some(Api::LinuxALSA),
            rtaudio_sys::RTAUDIO_API_UNIX_JACK => Some(Api::UnixJack),
            rtaudio_sys::RTAUDIO_API_LINUX_PULSE => Some(Api::LinuxPulse),
            rtaudio_sys::RTAUDIO_API_LINUX_OSS => Some(Api::LinuxOSS),
            rtaudio_sys::RTAUDIO_API_WINDOWS_ASIO => Some(Api::WindowsASIO),
            rtaudio_sys::RTAUDIO_API_WINDOWS_WASAPI => Some(Api::WindowsWASAPI),
            rtaudio_sys::RTAUDIO_API_WINDOWS_DS => Some(Api::WindowsDS),
            rtaudio_sys::RTAUDIO_API_DUMMY => Some(Api::Dummy),
            _ => None,
        }
    }

    pub fn to_raw(&self) -> rtaudio_sys::rtaudio_api_t {
        *self as rtaudio_sys::rtaudio_api_t
    }
}
