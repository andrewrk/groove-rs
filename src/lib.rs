#![allow(unstable)]
extern crate libc;

use libc::{c_int, uint64_t};

#[link(name="groove")]
extern {
    fn groove_init() -> c_int;
    fn groove_finish();
    fn groove_set_logging(level: c_int);
    fn groove_channel_layout_count(channel_layout: uint64_t) -> c_int;
    fn groove_channel_layout_default(count: c_int) -> uint64_t;
    fn groove_sample_format_bytes_per_sample(format: i32) -> c_int;
}

pub enum Log {
    Quiet,
    Error,
    Warning,
    Info,
}
impl Copy for Log {}


pub enum ChannelLayout {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LayoutMono,
    LayoutStereo,
}
impl Copy for ChannelLayout {}

const CH_FRONT_LEFT    :uint64_t = 0x00000001;
const CH_FRONT_RIGHT   :uint64_t = 0x00000002;
const CH_FRONT_CENTER  :uint64_t = 0x00000004;
const CH_LAYOUT_MONO   :uint64_t = CH_FRONT_CENTER;
const CH_LAYOUT_STEREO :uint64_t = CH_FRONT_LEFT|CH_FRONT_RIGHT;

impl ChannelLayout {
    /// get the default channel layout based on the channel count
    pub fn default(count: i32) -> Self {
        let x = unsafe { groove_channel_layout_default(count) };
        ChannelLayout::from_groove(x)
    }

    /// Get the channel count for the channel layout
    pub fn count(&self) -> i32 {
        unsafe { groove_channel_layout_count(self.to_groove()) as i32 }
    }

    fn to_groove(&self) -> uint64_t {
        match *self {
            ChannelLayout::FrontLeft    => CH_FRONT_LEFT,
            ChannelLayout::FrontRight   => CH_FRONT_RIGHT,
            ChannelLayout::FrontCenter  => CH_FRONT_CENTER,
            ChannelLayout::LayoutMono   => CH_LAYOUT_MONO,
            ChannelLayout::LayoutStereo => CH_LAYOUT_STEREO,
        }
    }

    fn from_groove(x: uint64_t) -> Self {
        match x {
            CH_FRONT_LEFT     => ChannelLayout::FrontLeft,
            CH_FRONT_RIGHT    => ChannelLayout::FrontRight,
            CH_FRONT_CENTER   => ChannelLayout::FrontCenter,
            CH_LAYOUT_STEREO  => ChannelLayout::LayoutStereo,
            _                 => panic!("invalid channel layout"),
        }
    }
}

const SAMPLE_FMT_NONE: i32 = -1;
const SAMPLE_FMT_U8:   i32 =  0;
const SAMPLE_FMT_S16:  i32 =  1;
const SAMPLE_FMT_S32:  i32 =  2;
const SAMPLE_FMT_FLT:  i32 =  3;
const SAMPLE_FMT_DBL:  i32 =  4;

const SAMPLE_FMT_U8P:  i32 =  5;
const SAMPLE_FMT_S16P: i32 =  6;
const SAMPLE_FMT_S32P: i32 =  7;
const SAMPLE_FMT_FLTP: i32 =  8;
const SAMPLE_FMT_DBLP: i32 =  9;

/// how to organize bits which represent audio samples
pub struct SampleFormat {
    sample_type: SampleType,
    /// planar means non-interleaved
    planar: bool,
}
impl Copy for SampleFormat {}

pub enum SampleType {
    NoType,
    /// unsigned 8 bits
    U8,
    /// signed 16 bits
    S16,
    /// signed 32 bits
    S32,
    /// float (32 bits)
    Flt,
    /// double (64 bits)
    Dbl,
}
impl Copy for SampleType {}

impl SampleFormat {
    fn to_groove(&self) -> i32 {
        match (self.sample_type, self.planar) {
            (SampleType::NoType, false) => SAMPLE_FMT_NONE,
            (SampleType::U8,     false) => SAMPLE_FMT_U8,
            (SampleType::S16,    false) => SAMPLE_FMT_S16,
            (SampleType::S32,    false) => SAMPLE_FMT_S32,
            (SampleType::Flt,    false) => SAMPLE_FMT_FLT,
            (SampleType::Dbl,    false) => SAMPLE_FMT_DBL,

            (SampleType::NoType, true)  => SAMPLE_FMT_NONE,
            (SampleType::U8,     true)  => SAMPLE_FMT_U8P,
            (SampleType::S16,    true)  => SAMPLE_FMT_S16P,
            (SampleType::S32,    true)  => SAMPLE_FMT_S32P,
            (SampleType::Flt,    true)  => SAMPLE_FMT_FLTP,
            (SampleType::Dbl,    true)  => SAMPLE_FMT_DBLP,
        }
    }

    pub fn bytes_per_sample(&self) -> i32 {
        unsafe { groove_sample_format_bytes_per_sample(self.to_groove()) }
    }
}

/// call once at the beginning of your program from the main thread
/// returns 0 on success, < 0 on error
pub fn init() -> isize {
    unsafe { groove_init() as isize }
}

/// call at the end of your program to clean up. after calling this
/// you may no longer use this API.
pub fn finish() {
    unsafe { groove_finish() }
}

/// enable/disable logging of errors
pub fn set_logging(level: Log) {
    let c_level: c_int = match level {
        Log::Quiet   => -8,
        Log::Error   => 16,
        Log::Warning => 24,
        Log::Info    => 32,
    };
    unsafe { groove_set_logging(c_level) }
}
