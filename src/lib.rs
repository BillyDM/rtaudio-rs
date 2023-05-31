use std::ffi::CStr;
use std::os::raw::c_char;

mod buffer;
mod device_info;
mod enums;
mod error;
mod host;
mod options;
mod stream;

pub use buffer::*;
pub use device_info::*;
pub use enums::*;
pub use error::*;
pub use host::*;
pub use options::*;
pub use stream::*;

/// Get the current RtAudio version.
pub fn version() -> String {
    // Safe because this C string will always be valid, we check
    // for the null case, and we don't free the pointer.
    unsafe {
        let raw_s = rtaudio_sys::rtaudio_version();
        if raw_s.is_null() {
            // I don't expect this to ever happen.
            return String::from("error");
        }

        CStr::from_ptr(raw_s as *mut c_char)
            .to_string_lossy()
            .to_string()
    }
}

/// Get the list of APIs compiled into this instance of RtAudio.
pub fn compiled_apis() -> Vec<Api> {
    // Safe because this list is gauranteed to be the reported length, we
    // check for the null case, and we do not free the `raw_list` pointer.
    let raw_apis_slice: &[rtaudio_sys::rtaudio_api_t] = unsafe {
        let num_compiled_apis = rtaudio_sys::rtaudio_get_num_compiled_apis();

        if num_compiled_apis == 0 {
            return Vec::new();
        }

        let raw_list = rtaudio_sys::rtaudio_compiled_api();

        if raw_list.is_null() {
            return Vec::new();
        }

        std::slice::from_raw_parts(raw_list, num_compiled_apis as usize)
    };

    raw_apis_slice
        .iter()
        .filter_map(|raw_api| Api::from_raw(*raw_api))
        .collect()
}
