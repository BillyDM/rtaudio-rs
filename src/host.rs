use crate::error::{RtAudioError, RtAudioErrorType};
use crate::{Api, DeviceInfo, DeviceParams, SampleFormat, Stream, StreamOptions};

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
        let show_int: i32 = if show { 1 } else { 0 };

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
        let id = unsafe { rtaudio_sys::rtaudio_get_device_id(self.raw, index as i32) };

        crate::check_for_error(self.raw)?;

        if id < 0 {
            return Err(RtAudioError {
                type_: RtAudioErrorType::InvalidDevice,
                msg: None,
            });
        }

        self.get_device_info_by_id(id as u32)
    }

    /// Retrieve info about an audio device by its ID.
    pub fn get_device_info_by_id(&self, id: u32) -> Result<DeviceInfo, RtAudioError> {
        // Safe because `self.raw` is gauranteed to not be null.
        let device_info_raw = unsafe { rtaudio_sys::rtaudio_get_device_info(self.raw, id as i32) };

        crate::check_for_error(self.raw)?;

        Ok(DeviceInfo::from_raw(device_info_raw))
    }

    /// Retrieve an iterator over the available audio devices.
    pub fn iter_devices<'a>(&'a self) -> DeviceIter<'a> {
        let num_devices = self.num_devices();
        DeviceIter {
            index: 0,
            num_devices,
            instance: self,
        }
    }

    /// Returns the device ID (not index) of the default output device.
    pub fn default_output_device_id(&self) -> u32 {
        // Safe because `self.raw` is gauranteed to not be null.
        unsafe { rtaudio_sys::rtaudio_get_default_output_device(self.raw) }
    }

    /// Returns the device ID (not index) of the default input device.
    pub fn default_input_device_id(&self) -> u32 {
        // Safe because `self.raw` is gauranteed to not be null.
        unsafe { rtaudio_sys::rtaudio_get_default_input_device(self.raw) }
    }

    /// Returns information about the default output device.
    pub fn default_output_device(&self) -> Result<DeviceInfo, RtAudioError> {
        let id = self.default_output_device_id();
        self.get_device_info_by_id(id)
    }

    /// Returns information about the default input device.
    pub fn default_input_device(&self) -> Result<DeviceInfo, RtAudioError> {
        let id = self.default_input_device_id();
        self.get_device_info_by_id(id)
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
    /// Only one stream can exist at a time.
    pub fn open_stream<E>(
        self,
        output_device: Option<DeviceParams>,
        input_device: Option<DeviceParams>,
        sample_format: SampleFormat,
        sample_rate: u32,
        buffer_frames: u32,
        options: StreamOptions,
        error_callback: E,
    ) -> Result<Stream, (Self, RtAudioError)>
    where
        E: FnOnce(RtAudioError) + Send + 'static,
    {
        Stream::new(
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
