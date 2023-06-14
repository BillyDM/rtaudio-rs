use std::ffi::CStr;
use std::os::raw::{c_int, c_uint, c_void};
use std::pin::Pin;
use std::sync::Mutex;

use crate::error::{RtAudioError, RtAudioErrorType};
use crate::{Buffers, DeviceParams, Host, SampleFormat, StreamFlags, StreamOptions, StreamStatus};

/// Information about a running RtAudio stream.
#[derive(Debug, Clone, Default)]
pub struct StreamInfo {
    /// The number of output audio channels.
    pub out_channels: usize,
    /// The number of input audio channels.
    pub in_channels: usize,

    /// The sample format.
    pub sample_format: SampleFormat,
    /// The sample rate.
    pub sample_rate: u32,

    /// The maximum number of frames that can appear in each call
    /// to `AudioCallback::process()`.
    pub max_frames: usize,

    /// Whether or not the buffers are interleaved (false), or
    /// deinterleaved (true).
    pub deinterleaved: bool,

    /// The internal latency in frames.
    ///
    /// If the API does not report latency, this will be `None`.
    pub latency: Option<usize>,

    /// The number of seconds that have elapsed since the stream was started.
    pub stream_time: f64,
}

/// An opened RtAudio stream.
///
/// When this struct is dropped, the stream will automatically be stopped
/// and closed.
///
/// Only one stream can exist at a time.
pub struct Stream {
    info: StreamInfo,
    raw: rtaudio_sys::rtaudio_t,
    started: bool,

    cb_context: Pin<Box<CallbackContext>>,
}

impl Stream {
    pub(crate) fn new<E>(
        mut host: Host,
        output_device: Option<DeviceParams>,
        input_device: Option<DeviceParams>,
        sample_format: SampleFormat,
        sample_rate: u32,
        buffer_frames: u32,
        options: StreamOptions,
        error_callback: E,
    ) -> Result<Stream, (Host, RtAudioError)>
    where
        E: FnOnce(RtAudioError) + Send + 'static,
    {
        assert!(!host.raw.is_null());
        let raw = host.raw;

        let mut raw_options = match options.to_raw() {
            Ok(o) => o,
            Err(e) => return Err((host, e)),
        };

        let mut info = StreamInfo {
            out_channels: output_device.map(|p| p.num_channels as usize).unwrap_or(0),
            in_channels: input_device.map(|p| p.num_channels as usize).unwrap_or(0),

            sample_format,
            sample_rate, // This will be overwritten later.

            max_frames: buffer_frames as usize, // This will be overwritten later.

            deinterleaved: options.flags.contains(StreamFlags::NONINTERLEAVED),

            latency: None, // This will be overwritten later.

            stream_time: 0.0,
        };

        let mut cb_context = Box::pin(CallbackContext {
            info: info.clone(),
            cb: Box::new(|_, _, _| {}), // This will be replaced later.
        });

        let cb_context_ptr: *mut CallbackContext = &mut *cb_context;

        let mut raw_output_device = output_device.map(|p| p.to_raw());
        let mut raw_input_device = input_device.map(|p| p.to_raw());

        let output_device_ptr: *mut rtaudio_sys::rtaudio_stream_parameters_t =
            if let Some(raw_output_device) = &mut raw_output_device {
                raw_output_device
            } else {
                std::ptr::null_mut()
            };
        let input_device_ptr: *mut rtaudio_sys::rtaudio_stream_parameters_t =
            if let Some(raw_input_device) = &mut raw_input_device {
                raw_input_device
            } else {
                std::ptr::null_mut()
            };

        {
            ERROR_CB_SINGLETON.lock().unwrap().cb = Some(Box::new(error_callback));
        }

        let mut buffer_frames_res = buffer_frames as c_uint;

        // Safe because we have checked that `raw` is not null, we have
        // constructed the `output_params` and `input_params` pointers
        // correctly, and we have pinned the `cb_context_ptr` pointer
        // in place. Also `cb_context_ptr` will always stay valid for
        // the lifetime the stream is open.
        unsafe {
            rtaudio_sys::rtaudio_open_stream(
                raw,
                output_device_ptr,
                input_device_ptr,
                sample_format.to_raw(),
                sample_rate as c_uint,
                &mut buffer_frames_res,
                Some(crate::stream::raw_data_callback),
                cb_context_ptr as *mut c_void,
                &mut raw_options,
                Some(raw_error_callback),
            )
        };
        if let Err(e) = crate::check_for_error(raw) {
            // Safe because we have checked that `raw` is not null.
            unsafe {
                rtaudio_sys::rtaudio_close_stream(raw);
            }
            {
                ERROR_CB_SINGLETON.lock().unwrap().cb = None;
            }
            return Err((host, e));
        }

        // Get info about the stream.
        info.max_frames = buffer_frames_res as usize;
        // Safe because we have checked that `raw` is not null.
        unsafe {
            let latency = rtaudio_sys::rtaudio_get_stream_latency(raw);
            if latency > 0 {
                info.latency = Some(latency as usize);
            }
        }
        if let Err(e) = crate::check_for_error(raw) {
            // Safe because we have checked that `raw` is not null.
            unsafe {
                rtaudio_sys::rtaudio_close_stream(raw);
            }
            {
                ERROR_CB_SINGLETON.lock().unwrap().cb = None;
            }
            return Err((host, e));
        }

        // Safe because we have checked that `raw` is not null.
        unsafe {
            let sr = rtaudio_sys::rtaudio_get_stream_sample_rate(raw);
            if sr > 0 {
                info.sample_rate = sr as u32;
            }
        };
        if let Err(e) = crate::check_for_error(raw) {
            // Safe because we have checked that `raw` is not null.
            unsafe {
                rtaudio_sys::rtaudio_close_stream(raw);
            }
            {
                ERROR_CB_SINGLETON.lock().unwrap().cb = None;
            }
            return Err((host, e));
        }

        cb_context.info = info.clone();

        let stream = Self {
            info,
            raw,
            started: false,
            cb_context,
        };

        // Make sure this isn't freed when `Host` is dropped.
        host.raw = std::ptr::null_mut();

        Ok(stream)
    }

    /// Information about the stream.
    pub fn info(&self) -> &StreamInfo {
        &self.info
    }

    /// Start the stream.
    ///
    /// * `data_callback` - This gets called whenever there are new buffers
    /// to process.
    ///
    /// If an error is returned, then it means that the stream failed to
    /// start.
    pub fn start<F>(&mut self, data_callback: F) -> Result<(), RtAudioError>
    where
        F: FnMut(Buffers<'_>, &StreamInfo, StreamStatus) + Send + 'static,
    {
        self.cb_context.cb = Box::new(data_callback);

        // Safe because `self.raw` cannot be null. Also, the data pointed to
        // the callback context is pinned in place, and it will always stay
        // valid for the lifetime that the stream is open.
        unsafe {
            rtaudio_sys::rtaudio_start_stream(self.raw);
        }
        if let Err(e) = crate::check_for_error(self.raw) {
            // Safe because `self.raw` cannot be null.
            unsafe {
                rtaudio_sys::rtaudio_stop_stream(self.raw);
            }

            return Err(e);
        }

        self.started = true;

        Ok(())
    }

    /// Stop the stream.
    ///
    /// This will block the calling thread until the stream is stopped. After
    /// which the `data_callback` passed into `Stream::start()` will be
    /// dropped.
    ///
    /// This does not close the stream.
    pub fn stop(&mut self) {
        if self.started {
            // Safe because `self.raw` cannot be null.
            unsafe { rtaudio_sys::rtaudio_stop_stream(self.raw) };
            if let Err(e) = crate::check_for_error(self.raw) {
                // TODO: Use log crate.
                eprintln!("{}", e);
            }

            // TODO: Make sure that the stream is always properly stopped
            // at this point.

            // Drop the user's callback.
            self.cb_context.cb = Box::new(|_, _, _| {});

            self.started = false;
        }
    }

    /// Close the stream.
    ///
    /// If the stream is running, this will stop the stream first. In that
    /// case, this will block the calling thread until the stream is stopped.
    /// After which the `data_callback` passed into `Stream::start()` will be
    /// dropped.
    pub fn close(mut self) -> Host {
        self.stop();

        // Safe because `self.raw` cannot be null.
        unsafe { rtaudio_sys::rtaudio_close_stream(self.raw) };
        if let Err(e) = crate::check_for_error(self.raw) {
            // TODO: use the log crate.
            eprintln!("{}", e);
        }

        let host = Host { raw: self.raw };

        // Make sure this isn't freed when `Stream` is dropped.
        self.raw = std::ptr::null_mut();

        host
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        {
            ERROR_CB_SINGLETON.lock().unwrap().cb = None;
        }

        if self.raw.is_null() {
            return;
        }

        self.stop();

        // Safe because we checked that `self.raw` is not null.
        unsafe { rtaudio_sys::rtaudio_close_stream(self.raw) };
        if let Err(e) = crate::check_for_error(self.raw) {
            // TODO: Use log crate.
            eprintln!("{}", e);
        }

        // Safe because we checked that `self.raw` is not null, and
        // we are guaranteed to be the only owner of this pointer.
        unsafe { rtaudio_sys::rtaudio_destroy(self.raw) };
    }
}

struct CallbackContext {
    info: StreamInfo,
    cb: Box<dyn FnMut(Buffers<'_>, &StreamInfo, StreamStatus) + Send + 'static>,
}

#[no_mangle]
pub(crate) unsafe extern "C" fn raw_data_callback(
    out: *mut c_void,
    in_: *mut c_void,
    frames: c_uint,
    stream_time: f64,
    status: rtaudio_sys::rtaudio_stream_status_t,
    userdata: *mut c_void,
) -> c_int {
    if userdata.is_null() {
        return 2;
    }
    if frames == 0 {
        return 0;
    }

    let cb_context_ptr = userdata as *mut CallbackContext;
    // Safe because we checked that this is not null. We have also
    // pinned this context in place, and it will always be valid for
    // the lifetime that this stream is open.
    let mut cb_context = unsafe { &mut *cb_context_ptr };

    cb_context.info.stream_time = stream_time;

    // This is safe because we assume that the correct amount
    // of data pointed to by `out` and `in_` exists. Also this
    // function checks if they are null.
    let buffers = unsafe {
        Buffers::from_raw(
            out,
            in_,
            frames as usize,
            cb_context.info.out_channels,
            cb_context.info.in_channels,
            cb_context.info.sample_format,
        )
    };

    let status = StreamStatus::from_bits_truncate(status);

    (cb_context.cb)(buffers, &cb_context.info, status);

    0
}

lazy_static::lazy_static! {
    static ref ERROR_CB_SINGLETON: Mutex<ErrorCallbackSingleton> =
        Mutex::new(ErrorCallbackSingleton { cb: None });
}

pub(crate) struct ErrorCallbackSingleton {
    cb: Option<Box<dyn FnOnce(RtAudioError) + Send + 'static>>,
}

#[no_mangle]
pub(crate) unsafe extern "C" fn raw_error_callback(
    raw_err: rtaudio_sys::rtaudio_error_t,
    raw_msg: *const ::std::os::raw::c_char,
) {
    if let Some(type_) = RtAudioErrorType::from_raw(raw_err) {
        if type_ == RtAudioErrorType::Warning {
            // Do nothing. While we could print the warning, we could be
            // in the realtime thread so it's better to not do that.
            return;
        }

        // Safe because this C string will always be valid, we check
        // for the null case, and we don't free the pointer.
        let msg = unsafe {
            if raw_msg.is_null() {
                None
            } else {
                let msg = CStr::from_ptr(raw_msg).to_string_lossy().to_string();

                if msg.is_empty() {
                    None
                } else {
                    Some(msg)
                }
            }
        };

        let e = RtAudioError { type_, msg };

        if let Some(cb) = { ERROR_CB_SINGLETON.lock().unwrap().cb.take() } {
            (cb)(e);
        }
    }
}
