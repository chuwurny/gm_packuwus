use std::{
    ffi::{c_char, CStr},
    ptr::null,
};

#[repr(C)]
#[derive(Debug)]
pub struct FileSystemVTable0 {
    _pad_1: [usize; 0x10],
    pub rename: unsafe extern "C" fn(
        *const *const FileSystemVTable0,
        *const c_char,
        *const c_char,
        *const c_char,
    ) -> bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct FileSystemVTable1 {
    _pad_1: [usize; 9],
    pub exists: unsafe extern "C" fn(
        *const *const FileSystemVTable1,
        *const c_char,
        *const c_char,
    ) -> bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct FileSystem {
    pub vtable_0: *const FileSystemVTable0,
    pub vtable_1: *const FileSystemVTable1,
}

#[derive(Debug)]
pub struct WrappedFileSystem(pub *const FileSystem);

impl WrappedFileSystem {
    pub fn exists(&self, filepath: &CStr, path_id: Option<&CStr>) -> bool {
        let path_id = if let Some(path_id) = path_id {
            path_id.as_ptr()
        } else {
            null()
        };

        unsafe {
            ((*(*self.0).vtable_1).exists)(
                &(*self.0).vtable_1,
                filepath.as_ptr(),
                path_id,
            )
        }
    }

    pub fn rename(&self, from: &CStr, to: &CStr, path_id: &CStr) -> bool {
        unsafe {
            ((*(*self.0).vtable_0).rename)(
                &(*self.0).vtable_0,
                from.as_ptr(),
                to.as_ptr(),
                path_id.as_ptr(),
            )
        }
    }
}
