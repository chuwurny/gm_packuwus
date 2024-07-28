use std::ffi::{c_char, c_void, CString, NulError};

use super::networkstringtable::{
    NetworkStringTable, WrappedNetworkStringTable,
};

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringTableContainerVTable {
    pub destructor_1: *const c_void,
    pub destructor_2: *const c_void,
    pub create_string_table: *const c_void,
    pub remove_all_tables: *const c_void,
    pub find_table: unsafe extern "C" fn(
        *const NetworkStringTableContainer,
        *const c_char,
    ) -> *const NetworkStringTable,
    pub table: *const c_void,
    pub num_tables: *const c_void,
    pub create_string_table_ex: *const c_void,
    pub set_allow_clientside_addstring: *const c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkStringTableContainer {
    pub vtable: *const NetworkStringTableContainerVTable,
}

#[derive(Debug)]
pub struct WrappedNetworkStringTableContainer(
    pub *const NetworkStringTableContainer,
);

impl<'a> WrappedNetworkStringTableContainer {
    pub fn find_table(
        &self,
        name: &str,
    ) -> Result<Option<WrappedNetworkStringTable>, NulError> {
        let name = CString::new(name)?;

        let table =
            unsafe { ((*(*self.0).vtable).find_table)(self.0, name.as_ptr()) };

        if !table.is_null() {
            Ok(Some(WrappedNetworkStringTable(table)))
        } else {
            Ok(None)
        }
    }
}
