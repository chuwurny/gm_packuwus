use std::{
    collections::HashMap,
    ffi::{CStr, CString, NulError},
    io::{Read, Write},
    mem::size_of,
    ptr::copy_nonoverlapping,
    string::FromUtf8Error,
};

use gmod::lua::{State, LUA_GLOBALSINDEX};
use gmod_lzma::SZ;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::sdk::{
    filesystem::{ReadFileError, WrappedFileSystem, WriteFileError},
    networkstringtable::WrappedNetworkStringTable,
};

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
pub enum HandlePackError {
    #[error("_G.PackUwUs_HandlePack is not defined")]
    NoGlobalFunc,
    #[error("Error occured in _G.PackUwUs_HandlePack")]
    LuaErrorOccured,
    #[error(
        "_G.PackUwUs_HandlePack returned invalid value (type: {0}). Valid types are: string, boolean"
    )]
    InvalidReturnValue(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AddFileError {
    #[error("Path contains \\0 character")]
    PathContainsNul(NulError),
    #[error("Failed to read file")]
    ReadFailed(ReadFileError),
    #[error("Failed to convert readed file contents as UTF-8: {0}")]
    FromUtf8Failed(FromUtf8Error),
    #[error("File already exists")]
    Exists,
}

#[derive(thiserror::Error, Debug)]
pub enum EditFileError {
    #[error("File doesn't exist")]
    DontExist,
}

#[derive(thiserror::Error, Debug)]
pub enum TryServeError {
    #[error("Failed to write packed file: {0}")]
    WriteFileFailed(WriteFileError),
    #[error("Packed contents is not set. Forgot to set it using PackUwUs_SetPackContent?")]
    PackedContentsNotSet,
}

#[derive(Debug)]
pub struct PackedFile {
    pub content: String,
}

#[derive(Debug)]
pub struct PackUwUs {
    lua: State,
    fs: WrappedFileSystem,
    downloadables: WrappedNetworkStringTable,
    client_lua_files: WrappedNetworkStringTable,
    files: HashMap<String, PackedFile>,
    pub content_changed: bool,
    pub packed_contents: Option<String>,
}

impl PackUwUs {
    pub fn new(
        lua: State,
        fs: WrappedFileSystem,
        downloadables: WrappedNetworkStringTable,
        client_lua_files: WrappedNetworkStringTable,
    ) -> PackUwUs {
        PackUwUs {
            lua,
            fs,
            downloadables,
            client_lua_files,
            files: HashMap::new(),
            content_changed: false,
            packed_contents: None,
        }
    }

    pub fn handle_pack(
        &self,
        filepath: &str,
        content: &str,
    ) -> Result<(bool, Option<String>), HandlePackError> {
        let mut should_pack = false;
        let mut new_content = None;

        unsafe {
            self.lua
                .get_field(LUA_GLOBALSINDEX, c"PackUwUs_HandlePack".as_ptr());

            if !self.lua.is_function(-1) {
                self.lua.pop(); // pop function

                return Err(HandlePackError::NoGlobalFunc);
            }

            self.lua.push_string(filepath);
            self.lua.push_string(content);

            if self.lua.pcall_ignore(2, 1) {
                if self.lua.is_boolean(-1) {
                    should_pack = self.lua.get_boolean(-1);
                } else if self.lua.get_type(-1) == "string" {
                    should_pack = true;
                    new_content = Some(self.lua.get_string(-1).unwrap().to_string());
                } else {
                    self.lua.pop(); // pop return value

                    return Err(HandlePackError::InvalidReturnValue(
                        self.lua.get_type(-1).to_string(),
                    ));
                }

                self.lua.pop(); // pop return value
            } else {
                self.lua.pop(); // pop function

                return Err(HandlePackError::LuaErrorOccured);
            }
        }

        Ok((should_pack, new_content))
    }

    pub fn add_file(
        &mut self,
        path: &str,
        new_content: Option<String>,
    ) -> Result<(), AddFileError> {
        if self.files.contains_key(path) {
            return Err(AddFileError::Exists);
        }

        let content = if let Some(new_content) = new_content {
            new_content
        } else {
            String::from_utf8(
                self.fs
                    .read_file(
                        CString::new(path)
                            .or_else(|err| Err(AddFileError::PathContainsNul(err)))?
                            .as_c_str(),
                        Some(c"GAME"),
                    )
                    .or_else(|err| Err(AddFileError::ReadFailed(err)))?,
            )
            .or_else(|err| Err(AddFileError::FromUtf8Failed(err)))?
        };

        self.content_changed = true;

        self.files.insert(path.to_string(), PackedFile { content });

        Ok(())
    }

    pub fn is_packed(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn edit_file(&mut self, path: &str, new_content: String) -> Result<(), EditFileError> {
        if let Some(packed_file) = self.files.get_mut(path) {
            self.content_changed = true;

            packed_file.content = new_content;

            Ok(())
        } else {
            Err(EditFileError::DontExist)
        }
    }

    fn pack(&self) -> Vec<u8> {
        let mut buf = vec![];

        for file in self.files.iter() {
            let filepath = file.0.as_bytes();
            let content = gmod_lzma::compress(file.1.content.as_bytes(), 9).unwrap();

            buf.reserve(filepath.len() + size_of::<usize>() + 1 + content.len());
            buf.write(filepath).unwrap();
            buf.write(&[0]).unwrap();
            buf.write(&content.len().to_le_bytes()).unwrap();
            buf.write(content.as_slice()).unwrap();
        }

        buf
    }

    pub fn try_serve(&mut self) -> Result<Option<String>, TryServeError> {
        if !self.content_changed {
            return Ok(None);
        }

        let hash = Uuid::new_v4().simple().to_string();
        let out_path = format!("data/serve_packuwus/{}.bsp", hash);

        println!("[PackUwUs] Writing {}", out_path);

        self.fs
            .write_file(
                CString::new(out_path.clone()).unwrap().as_c_str(),
                Some(c"GAME"),
                self.pack().as_slice(),
            )
            .or_else(|err| Err(TryServeError::WriteFileFailed(err)))?;

        // Update lua file hashes
        println!("[PackUwUs] Updating lua file hashes");

        for index in 0..self.client_lua_files.num_strings() {
            let filepath = self.client_lua_files.string(index);

            if let Some(filepath) = filepath {
                if self
                    .files
                    .contains_key(&filepath.to_string_lossy().to_string())
                {
                    let mut hash = Sha256::new();

                    hash.update(
                        CString::new(
                            self.packed_contents
                                .as_ref()
                                .ok_or_else(|| TryServeError::PackedContentsNotSet)?
                                .as_str(),
                        )
                        .unwrap()
                        .as_bytes_with_nul(),
                    );

                    let hash = hash.finalize();

                    self.client_lua_files
                        .set_string_userdata(index, hash.as_slice());
                }
            }
        }

        // Serve packed file
        println!("[PackUwUs] Serving packed file");

        let out_path_c_str = CString::new(out_path).unwrap();

        let mut downloadable_filepath = None;

        for index in 0..self.downloadables.num_strings() {
            let str = self.downloadables.string(index).unwrap();

            if str.to_string_lossy().starts_with("data/serve_packuwus/") {
                downloadable_filepath = Some((index, str));

                println!(
                    "[PackUwUs] Found old served packed file {}",
                    str.to_string_lossy()
                );

                break;
            }
        }

        if let Some(downloadable_filepath) = downloadable_filepath {
            unsafe {
                copy_nonoverlapping(
                    out_path_c_str.as_ptr(),
                    downloadable_filepath.1.as_ptr() as _,
                    out_path_c_str.count_bytes() + 1,
                );
            };
        } else {
            let index = self.downloadables.add_string(true, &out_path_c_str, None);

            println!(
                "[PackUwUs] Added new value to network string table (index: {})",
                index
            );
        }

        println!("[PackUwUs] Internal pack done!");

        Ok(Some(hash))
    }
}

impl PackUwUs {
    pub fn build_lua_download_packet(
        file_id: u16,
        lua_code: &str,
    ) -> Result<Vec<u8>, BuildLuaDownloadPacketError> {
        let lua_code = CString::new(lua_code)
            .or_else(|err| Err(BuildLuaDownloadPacketError::LuaCodeContainsNul(err)))?;

        let compressed_lua_code = gmod_lzma::compress(lua_code.as_bytes_with_nul(), 9)
            .or_else(|err| Err(BuildLuaDownloadPacketError::CompressFailed(err)))?;

        let lua_code_hash = {
            let mut hasher = Sha256::new();
            hasher.update(lua_code.as_bytes_with_nul());
            hasher.finalize().to_vec()
        };

        let mut new_data = vec![0; 1 + 2 + 0x20 + compressed_lua_code.len() + 1];

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
        let filepath = CString::new(filepath)
            .or_else(|err| Err(BuildLuaAutoRefreshPacketError::FilepathContainsNul(err)))?;

        let lua_code = CString::new(lua_code)
            .or_else(|err| Err(BuildLuaAutoRefreshPacketError::LuaCodeContainsNul(err)))?;

        let compressed_lua_code = gmod_lzma::compress(lua_code.as_bytes_with_nul(), 9)
            .or_else(|err| Err(BuildLuaAutoRefreshPacketError::CompressFailed(err)))?;

        let lua_code_hash = {
            let mut hasher = Sha256::new();
            hasher.update(lua_code.as_bytes_with_nul());
            hasher.finalize().to_vec()
        };

        let mut new_data =
            vec![
                0 as u8;
                1 + filepath.as_bytes_with_nul().len() + 4 + 0x20 + compressed_lua_code.len() + 1
            ];

        new_data[0] = 1;

        unsafe {
            let mut new_data_ptr = new_data.as_mut_ptr().byte_offset(1);

            copy_nonoverlapping(
                filepath.as_ptr(),
                new_data_ptr as _,
                filepath.as_bytes_with_nul().len(),
            );

            new_data_ptr = new_data_ptr.byte_offset(filepath.as_bytes_with_nul().len() as _);

            (new_data_ptr as *mut u32).write_unaligned((compressed_lua_code.len() as u32) + 0x20);

            new_data_ptr = new_data_ptr.byte_offset(4);

            copy_nonoverlapping(lua_code_hash.as_ptr(), new_data_ptr as _, 0x20);

            new_data_ptr = new_data_ptr.byte_offset(0x20);

            copy_nonoverlapping(
                compressed_lua_code.as_ptr(),
                new_data_ptr as _,
                compressed_lua_code.len(),
            );
        }

        Ok(new_data)
    }
}
