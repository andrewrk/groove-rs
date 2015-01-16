#![allow(unstable)]
extern crate libc;

use libc::{c_int};

#[link(name="groove")]
extern {
    fn groove_init() -> c_int;
    fn groove_finish();
}

pub fn init() -> isize {
    unsafe { groove_init() as isize }
}

pub fn finish() {
    unsafe { groove_finish() }
}
