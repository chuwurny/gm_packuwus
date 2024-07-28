use std::{
    ffi::{CString, NulError},
    ptr::copy_nonoverlapping,
};

use gmod::lua::{State, LUA_GLOBALSINDEX};
use gmod_lzma::SZ;
use sha2::{Digest, Sha256};

#[derive(thiserror::Error, Debug)]
pub enum BuildLuaDownloadPacketError {
    #[error("Lua code contains \\0 byte, what a mistake!")]
    LuaCodeContainsNul(NulError),
    #[error("Failed to compress lua code: {0}")]
    CompressFailed(SZ),
}

#[derive(thiserror::Error, Debug)]
pub enum BuildLuaAutoRefreshPacketError {
    #[error("File path contains \\0 byte")]
    FilepathContainsNul(NulError),
    #[error("Lua code contains \\0 byte, what a mistake!")]
    LuaCodeContainsNul(NulError),
    #[error("Failed to compress lua code: {0}")]
    CompressFailed(SZ),
}

#[derive(thiserror::Error, Debug)]
pub enum ShouldPackError {
    #[error("_G.PackUwUs_ShouldPack is not defined")]
    NoGlobalFunc,
    #[error("Error occured in _G.PackUwUs_ShouldPack")]
    LuaErrorOccured,
}

#[derive(thiserror::Error, Debug)]
pub enum ModifyContentError {
    #[error("_G.PackUwUs_ModifyContent is not defined")]
    NoGlobalFunc,
    #[error("Error occured in _G.PackUwUs_ModifyContent")]
    LuaErrorOccured,
}

#[derive(thiserror::Error, Debug)]
pub enum NotifyClientFileError {
    #[error("_G.PackUwUs_ClientFile is not defined")]
    NoGlobalFunc,
    #[error("Error occured in _G.PackUwUs_ClientFile")]
    LuaErrorOccured,
}

#[derive(Debug)]
pub struct PackUwUs {
    lua: State,
}

impl PackUwUs {
    pub fn new(lua: State) -> PackUwUs {
        PackUwUs { lua }
    }

    pub fn build_lua_download_packet(
        file_id: u16,
        lua_code: &str,
    ) -> Result<Vec<u8>, BuildLuaDownloadPacketError> {
        let lua_code = CString::new(lua_code).or_else(|err| {
            Err(BuildLuaDownloadPacketError::LuaCodeContainsNul(err))
        })?;

        let compressed_lua_code =
            gmod_lzma::compress(lua_code.as_bytes_with_nul(), 9).or_else(
                |err| Err(BuildLuaDownloadPacketError::CompressFailed(err)),
            )?;

        let lua_code_hash = {
            let mut hasher = Sha256::new();
            hasher.update(lua_code.as_bytes_with_nul());
            hasher.finalize().to_vec()
        };

        let mut new_data =
            vec![0; 1 + 2 + 0x20 + compressed_lua_code.len() + 1];

        new_data[0] = 4 /* GarrysMod::Networking::LuaDownloadFile */;

        unsafe {
            (new_data.as_mut_ptr() as *mut u16)
                .byte_offset(1)
                .write_unaligned(file_id);

            copy_nonoverlapping(
                lua_code_hash.as_ptr(),
                new_data.as_mut_ptr().byte_offset(3),
                0x20,
            );

            copy_nonoverlapping(
                compressed_lua_code.as_ptr(),
                new_data.as_mut_ptr().byte_offset(0x23),
                compressed_lua_code.len(),
            );
        }

        Ok(new_data)
    }

    pub fn build_lua_autorefresh_packet(
        filepath: &str,
        lua_code: &str,
    ) -> Result<Vec<u8>, BuildLuaAutoRefreshPacketError> {
        let filepath = CString::new(filepath).or_else(|err| {
            Err(BuildLuaAutoRefreshPacketError::FilepathContainsNul(err))
        })?;

        let lua_code = CString::new(lua_code).or_else(|err| {
            Err(BuildLuaAutoRefreshPacketError::LuaCodeContainsNul(err))
        })?;

        let compressed_lua_code =
            gmod_lzma::compress(lua_code.as_bytes_with_nul(), 9).or_else(
                |err| Err(BuildLuaAutoRefreshPacketError::CompressFailed(err)),
            )?;

        let lua_code_hash = {
            let mut hasher = Sha256::new();
            hasher.update(lua_code.as_bytes_with_nul());
            hasher.finalize().to_vec()
        };

        let mut new_data = vec![
            0 as u8;
            1 + filepath.as_bytes_with_nul().len()
                + 4
                + 0x20
                + compressed_lua_code.len()
                + 1
        ];

        new_data[0] = 1;

        unsafe {
            let mut new_data_ptr = new_data.as_mut_ptr().byte_offset(1);

            copy_nonoverlapping(
                filepath.as_ptr(),
                new_data_ptr as _,
                filepath.as_bytes_with_nul().len(),
            );

            new_data_ptr = new_data_ptr
                .byte_offset(filepath.as_bytes_with_nul().len() as _);

            (new_data_ptr as *mut u32)
                .write_unaligned((compressed_lua_code.len() as u32) + 0x20);

            new_data_ptr = new_data_ptr.byte_offset(4);

            copy_nonoverlapping(
                lua_code_hash.as_ptr(),
                new_data_ptr as _,
                0x20,
            );

            new_data_ptr = new_data_ptr.byte_offset(0x20);

            copy_nonoverlapping(
                compressed_lua_code.as_ptr(),
                new_data_ptr as _,
                compressed_lua_code.len(),
            );
        }

        Ok(new_data)
    }

    pub fn should_pack(
        &self,
        filepath: &str,
        reload: bool,
    ) -> Result<bool, ShouldPackError> {
        let mut should_pack = false;

        unsafe {
            self.lua
                .get_field(LUA_GLOBALSINDEX, c"PackUwUs_ShouldPack".as_ptr());

            if !self.lua.is_function(-1) {
                self.lua.pop(); // pop function

                return Err(ShouldPackError::NoGlobalFunc);
            }

            self.lua.push_string(filepath);
            self.lua.push_boolean(reload);

            if self.lua.pcall_ignore(2, 1) {
                if self.lua.is_boolean(-1) && self.lua.get_boolean(-1) {
                    should_pack = true;
                }

                self.lua.pop(); // pop return value
            } else {
                self.lua.pop(); // pop function

                return Err(ShouldPackError::LuaErrorOccured);
            }
        }

        Ok(should_pack)
    }

    pub fn modify_content(
        &self,
        filepath: &str,
        content: &str,
    ) -> Result<Option<String>, ModifyContentError> {
        unsafe {
            self.lua.get_field(
                LUA_GLOBALSINDEX,
                c"PackUwUs_ModifyContent".as_ptr(),
            );

            if !self.lua.is_function(-1) {
                self.lua.pop(); // pop function

                return Err(ModifyContentError::NoGlobalFunc);
            }

            self.lua.push_string(filepath);
            self.lua.push_string(content);

            if self.lua.pcall_ignore(2, 1) {
                if self.lua.get_type(-1) == "string" {
                    let new_content =
                        self.lua.get_string(-1).unwrap().to_string();

                    self.lua.pop(); // pop new content

                    return Ok(Some(new_content));
                }

                self.lua.pop(); // pop return value
            } else {
                self.lua.pop(); // pop function

                return Err(ModifyContentError::LuaErrorOccured);
            }

            Ok(None)
        }
    }

    pub fn notify_client_file(
        &self,
        filepath: &str,
        reload: bool,
    ) -> Result<(), NotifyClientFileError> {
        unsafe {
            self.lua
                .get_field(LUA_GLOBALSINDEX, c"PackUwUs_ClientFile".as_ptr());

            if !self.lua.is_function(-1) {
                self.lua.pop(); // pop function

                return Err(NotifyClientFileError::NoGlobalFunc);
            }

            self.lua.push_string(filepath);
            self.lua.push_boolean(reload);

            if !self.lua.pcall_ignore(2, 0) {
                self.lua.pop(); // pop function

                return Err(NotifyClientFileError::LuaErrorOccured);
            }

            Ok(())
        }
    }
}
