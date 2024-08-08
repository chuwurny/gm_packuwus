use std::{
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    ptr::null,
    string::FromUtf8Error,
};

pub type FileHandle = *const c_void;

pub const INVALID_FILE_HANDLE: FileHandle = null();

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
    pub read: unsafe extern "C" fn(
        *const *const FileSystemVTable1,
        *mut c_void,
        c_int,
        FileHandle,
    ) -> c_int,
    pub write: unsafe extern "C" fn(
        *const *const FileSystemVTable1,
        *const c_void,
        c_int,
        FileHandle,
    ) -> c_int,
    pub open: unsafe extern "C" fn(
        *const *const FileSystemVTable1,
        *const c_char,
        *const c_char,
        *const c_char,
    ) -> FileHandle,
    pub close: unsafe extern "C" fn(*const *const FileSystemVTable1, FileHandle) -> FileHandle,
    _pad_2: [usize; 2],
    pub size: unsafe extern "C" fn(*const *const FileSystemVTable1, FileHandle) -> c_uint,
    _pad_3: [usize; 2],
    pub exists:
        unsafe extern "C" fn(*const *const FileSystemVTable1, *const c_char, *const c_char) -> bool,
}

#[derive(thiserror::Error, Debug)]
pub enum ReadFileError {
    #[error("Failed to open file")]
    OpenFailed,
}

#[derive(thiserror::Error, Debug)]
pub enum WriteFileError {
    #[error("Failed to open file")]
    OpenFailed,
}

#[repr(C)]
#[derive(Debug)]
pub struct FileSystem {
    pub vtable_0: *const FileSystemVTable0,
    pub vtable_1: *const FileSystemVTable1,
}

#[derive(Debug, Clone, Copy)]
pub struct WrappedFileSystem(pub *const FileSystem);

impl WrappedFileSystem {
    pub fn read_file(
        &self,
        filepath: &CStr,
        path_id: Option<&CStr>,
    ) -> Result<Vec<u8>, ReadFileError> {
        let path_id = if let Some(path_id) = path_id {
            path_id.as_ptr()
        } else {
            null()
        };

        let handle = unsafe {
            ((*(*self.0).vtable_1).open)(
                &(*self.0).vtable_1,
                filepath.as_ptr(),
                c"rb".as_ptr(),
                path_id,
            )
        };

        if handle == INVALID_FILE_HANDLE {
            return Err(ReadFileError::OpenFailed);
        }

        let size = unsafe { ((*(*self.0).vtable_1).size)(&(*self.0).vtable_1, handle) };

        let mut buf = vec![0; size as _];

        unsafe {
            ((*(*self.0).vtable_1).read)(
                &(*self.0).vtable_1,
                buf.as_mut_ptr() as _,
                size as _,
                handle,
            )
        };

        unsafe { ((*(*self.0).vtable_1).close)(&(*self.0).vtable_1, handle) };

        Ok(buf)
    }

    pub fn write_file(
        &self,
        filepath: &CStr,
        path_id: Option<&CStr>,
        content: &[u8],
    ) -> Result<(), WriteFileError> {
        let path_id = if let Some(path_id) = path_id {
            path_id.as_ptr()
        } else {
            null()
        };

        let handle = unsafe {
            ((*(*self.0).vtable_1).open)(
                &(*self.0).vtable_1,
                filepath.as_ptr(),
                c"wb".as_ptr(),
                path_id,
            )
        };

        if handle == INVALID_FILE_HANDLE {
            return Err(WriteFileError::OpenFailed);
        }

        unsafe {
            ((*(*self.0).vtable_1).write)(
                &(*self.0).vtable_1,
                content.as_ptr() as _,
                content.len() as _,
                handle,
            )
        };

        unsafe { ((*(*self.0).vtable_1).close)(&(*self.0).vtable_1, handle) };

        Ok(())
    }

    pub fn exists(&self, filepath: &CStr, path_id: Option<&CStr>) -> bool {
        let path_id = if let Some(path_id) = path_id {
            path_id.as_ptr()
        } else {
            null()
        };

        unsafe { ((*(*self.0).vtable_1).exists)(&(*self.0).vtable_1, filepath.as_ptr(), path_id) }
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
