use std::ffi::{c_uint, c_void};

#[repr(C)]
#[derive(Debug)]
pub struct AutoBuffer {
    pub _unk_1: *const c_void,
    pub data: *const c_void,
    pub _unk_2: *const c_void,
    pub pos: c_uint,
    pub written: c_uint,
}
