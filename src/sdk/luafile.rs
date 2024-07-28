use std::{
    ffi::{c_void, CStr},
    fmt::Display,
};

use super::bootil::buffer::AutoBuffer;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LuaFileString(*const c_void);

impl<'a> Into<&'a CStr> for LuaFileString {
    fn into(self) -> &'a CStr {
        unsafe { CStr::from_ptr(self.0 as _) }
    }
}

impl<'a> LuaFileString {
    pub fn as_c_str(&self) -> &'a CStr {
        Into::<&CStr>::into(*self)
    }
}

impl Display for LuaFileString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            Into::<&CStr>::into(*self)
                .to_string_lossy()
                .into_owned()
                .as_str(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LuaFileContent(*const c_void);

impl LuaFileContent {
    // returns 0 if file is empty. perhaps it's a length?
    pub fn empty_indicator(&self) -> i32 {
        /*
        if (*(int *)((int)luaFile->m_pFileContent + -0xc) == 0) {
          (**(code **)((int)g_Lua->vmt + 0x1c4))
                    (g_Lua,"AddCSLuaFile: Empty file \'%s\'\n",luaFile-szFilepath// );
          goto LAB_009e6d8a;
        }
        */
        unsafe { *(*(self.0 as *const *const i32).offset(-3)) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LuaFile {
    pub _unk_1: *const c_void,
    pub name: LuaFileString,
    pub kind_of: LuaFileString,
    pub content: LuaFileString,
    pub _unk_2: *const AutoBuffer,
    pub _unk_3: i32,
    pub _unk_4: i32,
}
