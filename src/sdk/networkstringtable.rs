use std::{
    ffi::{c_char, c_int, c_void, CStr},
    ptr::null,
};

use super::networkstringdict::{NetworkStringDict, WrappedNetworkStringDict};

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringTableVTable {
    pub destructor_1: *const c_void,
    pub destructor_2: *const c_void,
    pub table_name: *const c_void,
    pub table_id: *const c_void,
    pub num_strings: unsafe extern "C" fn(*const NetworkStringTable) -> c_int,
    pub max_strings: *const c_void,
    pub entry_bits: *const c_void,
    pub set_tick: *const c_void,
    pub changed_since_tick: *const c_void,
    pub add_string: unsafe extern "C" fn(
        *const NetworkStringTable,
        bool,
        *const c_char,
        c_int,
        *const c_void,
    ) -> c_int,
    pub string:
        unsafe extern "C" fn(*const NetworkStringTable, c_int) -> *const c_char,
    pub set_string_userdata: *const c_void,
    pub string_userdata: *const c_void,
    pub find_string_index: *const c_void,
    pub set_string_changed_callback: *const c_void,
    pub dump: *const c_void,
    pub lock: *const c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringTable {
    pub vtable: *const NetworkStringTableVTable,
    _pad_1: [usize; 0x4],
    pub name: *const c_char,
    pub max_strings: c_int,
    _pad_2: [usize; 0x24],
    pub items: *const NetworkStringDict,
    pub items_clientside: *const NetworkStringDict,
}

#[derive(Debug)]
pub struct WrappedNetworkStringTable(pub *const NetworkStringTable);

impl<'a> WrappedNetworkStringTable {
    pub fn num_strings(&self) -> i32 {
        unsafe { ((*(*self.0).vtable).num_strings)(self.0) }
    }

    pub fn string(&self, index: i32) -> Option<&'a CStr> {
        let str = unsafe { ((*(*self.0).vtable).string)(self.0, index) };

        if !str.is_null() {
            Some(unsafe { CStr::from_ptr(str) })
        } else {
            None
        }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self.0).name) }
    }

    pub fn items(&self) -> WrappedNetworkStringDict {
        unsafe { WrappedNetworkStringDict((*self.0).items) }
    }

    pub fn add_string(&self, server: bool, value: &CStr) -> i32 {
        unsafe {
            ((*(*self.0).vtable).add_string)(
                self.0,
                server,
                value.as_ptr(),
                -1,
                null(),
            )
        }
    }
}
