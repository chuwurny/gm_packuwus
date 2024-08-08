use std::ffi::{c_char, c_int, c_void, CStr};

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringDictVTable {
    pub destructor_1: *const c_void,
    pub destructor_2: *const c_void,
    pub count: *const c_void,
    pub purge: *const c_void,
    pub string: unsafe extern "C" fn(*const NetworkStringDict, c_int) -> *const c_char,
    pub is_valid_index: unsafe extern "C" fn(*const NetworkStringDict, c_int) -> bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringDict {
    pub vtable: *const NetworkStringDictVTable,
}

#[derive(Debug, Clone, Copy)]
pub struct WrappedNetworkStringDict(pub *const NetworkStringDict);

impl WrappedNetworkStringDict {
    pub fn string(&self, index: i32) -> Option<&CStr> {
        let str = unsafe { ((*(*self.0).vtable).string)(self.0, index) };

        if !str.is_null() {
            Some(unsafe { CStr::from_ptr(str) })
        } else {
            None
        }
    }

    pub fn is_valid_index(&self, index: i32) -> bool {
        unsafe { ((*(*self.0).vtable).is_valid_index)(self.0, index) }
    }
}
