#![allow(unstable)]
extern crate libc;

use libc::{c_int};

pub enum Log {
    Quiet,
    Error,
    Warning,
    Info,
}

impl Copy for Log {}

#[link(name="groove")]
extern {
    fn groove_init() -> c_int;
    fn groove_finish();
    fn groove_set_logging(level: c_int);
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
