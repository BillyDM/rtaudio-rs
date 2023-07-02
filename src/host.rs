use crate::error::{RtAudioError, RtAudioErrorType};
use crate::{Api, DeviceID, DeviceInfo, DeviceParams, SampleFormat, StreamHandle, StreamOptions};
use std::os::raw::{c_int, c_uint};

/// An RtAudio Host instance. This is used to enumerate audio devices before
/// opening a stream.
#[derive(Debug)]
pub struct Host {
    pub(crate) raw: rtaudio_sys::rtaudio_t,
}

impl Host {
    /// Create a new RtAudio Host with the given API. This host is used to
    /// enumerate audio devices before opening a stream.
    ///
    /// If `Api::Unspecified` is used, then the best one for the system will
    /// automatically be chosen.
    pub fn new(api: Api) -> Result<Self, RtAudioError> {
        // Safe because we check for the null case.
        let raw = unsafe { rtaudio_sys::rtaudio_create(api.to_raw()) };

        if raw.is_null() {
            return Err(RtAudioError {
                type_: RtAudioErrorType::Unkown,
                msg: Some("failed to create RtAudio instance".into()),
            });
        }

        let new_self = Self { raw };

        crate::check_for_error(new_self.raw)?;

        Ok(new_self)
    }

    /// Whether or not to print extra warnings to the terminal output.
    ///
    /// By default this is set to `false`.
    pub fn show_warnings(&self, show: bool) {
        let show_int: c_int = if show { 1 } else { 0 };

        unsafe {
            rtaudio_sys::rtaudio_show_warnings(self.raw, show_int);
        }
    }

    /// The API being used by this instance.
    pub fn api(&self) -> Api {
        // Safe because `self.raw` is gauranteed to not be null.
        let api_raw = unsafe { rtaudio_sys::rtaudio_current_api(self.raw) };
        Api::from_raw(api_raw).unwrap_or(Api::Unspecified)
    }

    /// Retrieve the number of available audio devices.
    pub fn num_devices(&self) -> usize {
        // Safe because `self.raw` is gauranteed to not be null.
        let num_devices = unsafe { rtaudio_sys::rtaudio_device_count(self.raw) };

        num_devices.max(0) as usize
    }

    /// Retrieve information about an audio device by its index.
    pub fn get_device_info_by_index(&self, index: usize) -> Result<DeviceInfo, RtAudioError> {
        // Safe because `self.raw` is gauranteed to not be null.
        let id = unsafe { rtaudio_sys::rtaudio_get_device_id(self.raw, index as c_int) };

        if id == 0 {
            return Err(RtAudioError {
                type_: RtAudioErrorType::InvalidParamter,
                msg: Some(format!("Could not find device at index {}", index)),
            });
        }

        crate::check_for_error(self.raw)?;

        self.get_device_info_by_id(DeviceID(id as u32))
    }

    /// Retrieve info about an audio device by its ID.
    pub fn get_device_info_by_id(&self, id: DeviceID) -> Result<DeviceInfo, RtAudioError> {
        // Safe because `self.raw` is gauranteed to not be null.
        let device_info_raw =
            unsafe { rtaudio_sys::rtaudio_get_device_info(self.raw, id.0 as c_uint) };

        crate::check_for_error(self.raw)?;

        Ok(DeviceInfo::from_raw(device_info_raw))
    }

    /// Retrieve an iterator over all the available audio devices (including ones
    /// that have failed to scan properly).
    pub fn iter_devices_complete<'a>(&'a self) -> DeviceIter<'a> {
        let num_devices = self.num_devices();
        DeviceIter {
            index: 0,
            num_devices,
            instance: self,
        }
    }

    /// Retrieve an iterator over the available audio devices.
    ///
    /// If there was a problem scanning a device, a warning will be printed
    /// to the log.
    pub fn iter_devices<'a>(&'a self) -> impl Iterator<Item = DeviceInfo> + 'a {
        self.iter_devices_complete().filter_map(|d| match d {
            Ok(d) => Some(d),
            Err(e) => {
                log::warn!("{}", e);

                None
            }
        })
    }

    /// Retrieve an iterator over the available output audio devices.
    ///
    /// If there was a problem scanning a device, a warning will be printed
    /// to the log.
    pub fn iter_output_devices<'a>(&'a self) -> impl Iterator<Item = DeviceInfo> + 'a {
        self.iter_devices_complete().filter_map(|d| match d {
            Ok(d) => {
                if d.output_channels > 0 {
                    Some(d)
                } else {
                    None
                }
            }
            Err(e) => {
                log::warn!("{}", e);

                None
            }
        })
    }

    /// Retrieve an iterator over the available input audio devices.
    ///
    /// If there was a problem scanning a device, a warning will be printed
    /// to the log.
    pub fn iter_input_devices<'a>(&'a self) -> impl Iterator<Item = DeviceInfo> + 'a {
        self.iter_devices_complete().filter_map(|d| match d {
            Ok(d) => {
                if d.input_channels > 0 {
                    Some(d)
                } else {
                    None
                }
            }
            Err(e) => {
                log::warn!("{}", e);

                None
            }
        })
    }

    /// Retrieve an iterator over the available duplex audio devices.
    ///
    /// If there was a problem scanning a device, a warning will be printed
    /// to the log.
    pub fn iter_duplex_devices<'a>(&'a self) -> impl Iterator<Item = DeviceInfo> + 'a {
        self.iter_devices_complete().filter_map(|d| match d {
            Ok(d) => {
                if d.duplex_channels > 0 {
                    Some(d)
                } else {
                    None
                }
            }
            Err(e) => {
                log::warn!("{}", e);

                None
            }
        })
    }

    /*
    /// Retrieve a list of available audio devices.
    pub fn devices(&self) -> Vec<DeviceInfo> {
        self.iter_devices().collect()
    }

    /// Retrieve a list of available output audio devices.
    pub fn output_devices(&self) -> Vec<DeviceInfo> {
        self.iter_output_devices().collect()
    }

    /// Retrieve a list of available input audio devices.
    pub fn input_devices(&self) -> Vec<DeviceInfo> {
        self.iter_input_devices().collect()
    }

    /// Retrieve a list of available duplex audio devices.
    pub fn duplex_devices(&self) -> Vec<DeviceInfo> {
        self.iter_duplex_devices().collect()
    }
    */

    /// Returns the device ID (not index) of the default output device.
    pub fn default_output_device_id(&self) -> Option<DeviceID> {
        // Safe because `self.raw` is gauranteed to not be null.
        let res = unsafe { rtaudio_sys::rtaudio_get_default_output_device(self.raw) };

        if res == 0 {
            None
        } else {
            Some(DeviceID(res as u32))
        }
    }

    /// Returns the device ID (not index) of the default input device.
    pub fn default_input_device_id(&self) -> Option<DeviceID> {
        // Safe because `self.raw` is gauranteed to not be null.
        let res = unsafe { rtaudio_sys::rtaudio_get_default_input_device(self.raw) };

        if res == 0 {
            None
        } else {
            Some(DeviceID(res as u32))
        }
    }

    /// Returns information about the default output device.
    pub fn default_output_device(&self) -> Result<DeviceInfo, RtAudioError> {
        if let Some(id) = self.default_output_device_id() {
            self.get_device_info_by_id(id)
        } else {
            Err(RtAudioError {
                type_: RtAudioErrorType::NoDevicesFound,
                msg: Some("No default output device found".into()),
            })
        }
    }

    /// Returns information about the default input device.
    pub fn default_input_device(&self) -> Result<DeviceInfo, RtAudioError> {
        if let Some(id) = self.default_input_device_id() {
            self.get_device_info_by_id(id)
        } else {
            Err(RtAudioError {
                type_: RtAudioErrorType::NoDevicesFound,
                msg: Some("No default input device found".into()),
            })
        }
    }

    /// Open a new audio stream.
    ///
    /// * `output_device` - The parameters for the output device to use. If you do
    /// not wish to use an output device, set this to `None`.
    /// * `input_device` - The parameters for the input device to use. If you do not
    /// wish to use an input device, set this to `None`.
    /// * `sample_format` - The sample format to use. If the device doesn't natively
    /// support the given format, then it will automatically be converted to/from
    /// that format.
    /// * `sample_rate` - The sample rate to use. The stream may decide to use a
    /// different sample rate if it's not supported.
    /// * `buffer_frames` - The desired maximum number of frames that can appear in a
    /// single process call. The stream may decide to use a different value if it's
    /// not supported. The given value should be a power of 2.
    /// * `options` - Additional options for the stream.
    /// * `error_callback` - This will be called if there was an error that caused the
    /// stream to close. If this happens, the returned `Stream` struct should be
    /// manually closed or dropped.
    ///
    /// Only one stream can be opened at a time (this is a limitation with RtAudio).
    pub fn open_stream<E>(
        self,
        output_device: Option<DeviceParams>,
        input_device: Option<DeviceParams>,
        sample_format: SampleFormat,
        sample_rate: u32,
        buffer_frames: u32,
        options: StreamOptions,
        error_callback: E,
    ) -> Result<StreamHandle, (Self, RtAudioError)>
    where
        E: FnOnce(RtAudioError) + Send + 'static,
    {
        StreamHandle::new(
            self,
            output_device,
            input_device,
            sample_format,
            sample_rate,
            buffer_frames,
            options,
            error_callback,
        )
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // Safe because we checked that the pointer is not null, and we
            // are guaranteed to be the only owner of this pointer.
            unsafe {
                rtaudio_sys::rtaudio_destroy(self.raw);
            }
        }
    }
}

pub struct DeviceIter<'a> {
    index: usize,
    num_devices: usize,

    instance: &'a Host,
}

impl<'a> Iterator for DeviceIter<'a> {
    type Item = Result<DeviceInfo, RtAudioError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index > self.num_devices {
            None
        } else {
            Some(self.instance.get_device_info_by_index(self.index - 1))
        }
    }
}
