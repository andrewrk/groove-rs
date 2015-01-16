#![allow(unstable)]
extern crate libc;

use libc::{c_int, uint64_t};

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
    fn to_groove(&self) -> uint64_t {
        match *self {
            ChannelLayout::FrontLeft    => CH_FRONT_LEFT,
            ChannelLayout::FrontRight   => CH_FRONT_RIGHT,
            ChannelLayout::FrontCenter  => CH_FRONT_CENTER,
            ChannelLayout::LayoutMono   => CH_LAYOUT_MONO,
            ChannelLayout::LayoutStereo => CH_LAYOUT_STEREO,
        }
    }
    pub fn count(&self) -> i32 {
        unsafe { groove_channel_layout_count(self.to_groove()) as i32 }
    }
}


#[link(name="groove")]
extern {
    fn groove_init() -> c_int;
    fn groove_finish();
    fn groove_set_logging(level: c_int);
    fn groove_channel_layout_count(channel_layout: uint64_t) -> c_int;
}

pub fn init() -> isize {
    unsafe { groove_init() as isize }
}

pub fn finish() {
    unsafe { groove_finish() }
}

pub fn set_logging(level: Log) {
    let c_level: c_int = match level {
        Log::Quiet   => -8,
        Log::Error   => 16,
        Log::Warning => 24,
        Log::Info    => 32,
    };
    unsafe { groove_set_logging(c_level) }
}
