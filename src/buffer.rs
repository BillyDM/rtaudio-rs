use std::ffi::c_void;

use crate::SampleFormat;

/// The input/output audio buffers.
#[derive(Debug, PartialEq)]
pub enum Buffers<'a> {
    /// Input/output buffers of 8-bit signed integers.
    SInt8 {
        output: &'a mut [i8],
        input: &'a [i8],
    },
    /// Input/output buffers of 16-bit signed integers.
    SInt16 {
        output: &'a mut [i16],
        input: &'a [i16],
    },
    /// Input/output buffers of 24-bit signed integers.
    ///
    /// These buffers are presented as raw bytes. Each sample in a
    /// frame is 3 bytes.
    ///
    /// The endianness will always be in the host's native byte order.
    SInt24 {
        output: &'a mut [u8],
        input: &'a [u8],
    },
    /// Input/output buffers of 32-bit signed integers.
    SInt32 {
        output: &'a mut [i32],
        input: &'a [i32],
    },
    /// Input/output buffers of 32-bit floating point numbers.
    Float32 {
        output: &'a mut [f32],
        input: &'a [f32],
    },
    /// Input/output buffers of 64-bit floating point numbers.
    Float64 {
        output: &'a mut [f64],
        input: &'a [f64],
    },
}

impl<'a> Buffers<'a> {
    pub(crate) unsafe fn from_raw(
        out: *mut c_void,
        in_: *mut c_void,
        frames: usize,
        out_channels: usize,
        in_channels: usize,
        sample_format: SampleFormat,
    ) -> Self {
        match sample_format {
            SampleFormat::SInt8 => {
                let out_ptr = out as *mut i8;
                let in_ptr = in_ as *const i8;

                let output: &'a mut [i8] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames)
                };
                let input: &'a [i8] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames)
                };

                Buffers::SInt8 { output, input }
            }
            SampleFormat::SInt16 => {
                let out_ptr = out as *mut i16;
                let in_ptr = in_ as *const i16;

                let output: &'a mut [i16] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames)
                };
                let input: &'a [i16] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames)
                };

                Buffers::SInt16 { output, input }
            }
            SampleFormat::SInt24 => {
                let out_ptr = out as *mut u8;
                let in_ptr = in_ as *const u8;

                let output: &'a mut [u8] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames * 3)
                };
                let input: &'a [u8] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames * 3)
                };

                Buffers::SInt24 { output, input }
            }
            SampleFormat::SInt32 => {
                let out_ptr = out as *mut i32;
                let in_ptr = in_ as *const i32;

                let output: &'a mut [i32] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames)
                };
                let input: &'a [i32] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames)
                };

                Buffers::SInt32 { output, input }
            }
            SampleFormat::Float32 => {
                let out_ptr = out as *mut f32;
                let in_ptr = in_ as *const f32;

                let output: &'a mut [f32] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames)
                };
                let input: &'a [f32] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames)
                };

                Buffers::Float32 { output, input }
            }
            SampleFormat::Float64 => {
                let out_ptr = out as *mut f64;
                let in_ptr = in_ as *const f64;

                let output: &'a mut [f64] = if out_ptr.is_null() || out_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(out_ptr, out_channels * frames)
                };
                let input: &'a [f64] = if in_ptr.is_null() || in_channels == 0 {
                    &mut []
                } else {
                    std::slice::from_raw_parts(in_ptr, in_channels * frames)
                };

                Buffers::Float64 { output, input }
            }
        }
    }
}
