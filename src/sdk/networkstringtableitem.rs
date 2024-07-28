use std::ffi::{c_int, c_void};

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringTableItem {
    pub user_data: *const c_void,
    pub user_data_len: c_int,
    pub tick_changed: c_int,
    // TODO: more ?
}
