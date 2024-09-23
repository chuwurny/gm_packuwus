use core::slice;
use std::ffi::{c_char, c_int, c_void, CStr, CString};

use retour::static_detour;

use crate::{packuwus::PackUwUs, sdk::luafile::LuaFile, CLIENT_FILES_TABLE, PACKUWUS};

static_detour! {
    pub(crate) static GMODDATAPACK_ADDORUPDATEFILE: unsafe extern "C" fn(*const c_void, *mut LuaFile, bool);
    pub(crate) static GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA: unsafe extern "C" fn(*const *const c_char, *const *const c_char, *const *const c_char) -> c_int;
    pub(crate) static CVENGINESERVER_GMOD_SENDTOCLIENT: unsafe extern "C" fn (*const c_void, i32, *const c_void, i32);
    pub(crate) static CVENGINESERVER_GMOD_SENDTOCLIENTS: unsafe extern "C" fn (*const c_void, *const c_void, *const c_void, i32);
}

pub(crate) fn new_gmoddatapack_addorupdatefile(
    this: *const c_void,
    file: *mut LuaFile,
    reload: bool,
) {
    #[cfg(debug_assertions)]
    println!(
        "GModDataPack::AddOrUpdateFile({:?}, {:?} ({}), {})",
        this,
        &file,
        Into::<&CStr>::into(unsafe { (*file).name })
            .to_str()
            .unwrap(),
        reload
    );

    unsafe {
        let path = (*file).name.as_c_str().to_str().unwrap();

        match PACKUWUS
            .as_ref()
            .unwrap()
            .handle_pack(path, &(*file).content.as_c_str().to_string_lossy())
        {
            Ok((should_pack, new_content)) => {
                if should_pack {
                    if reload {
                        if let Err(err) = PACKUWUS.as_mut().unwrap().edit_file(
                            path,
                            new_content.unwrap_or_else(|| (*file).content.to_string()),
                        ) {
                            println!("[PackUwUs] Failed to edit file {}", err);
                        }
                    } else {
                        if let Err(err) = PACKUWUS.as_mut().unwrap().add_file(path, new_content) {
                            println!("[PackUwUs] Failed to add file: {}", err);
                        }
                    }
                }
            }
            Err(err) => println!("[PackUwUs] Failed to notify client file: {}", err),
        }
    }

    unsafe { GMODDATAPACK_ADDORUPDATEFILE.call(this, file, reload) }
}

pub(crate) fn new_garrysmod_autorefresh_handlechange_lua(
    directory: *const *const c_char,
    filename: *const *const c_char,
    file_ext: *const *const c_char,
) -> c_int {
    #[cfg(debug_assertions)]
    println!(
        "GarrysMod::AutoRefresh::HandleChange_Lua({:?}, {:?}, {:?})",
        directory, filename, file_ext
    );

    #[cfg(debug_assertions)]
    unsafe {
        dbg!(CStr::from_ptr(*directory).to_string_lossy());
        dbg!(CStr::from_ptr(*filename).to_string_lossy());
        dbg!(CStr::from_ptr(*file_ext).to_string_lossy());
    }

    unsafe { GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA.call(directory, filename, file_ext) }
}

pub(crate) fn new_cvengineserver_gmod_sendtoclient(
    this: *const c_void,
    client_id: i32,
    data: *const c_void,
    data_len: i32,
) {
    #[cfg(debug_assertions)]
    println!(
        "CVEngineServer::GMOD_SendToClient({:?}, {}, {:?}, {})",
        this, client_id, data, data_len
    );

    unsafe fn build_download_packet(
        file_id: u16,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        if !PACKUWUS.as_ref().unwrap().is_packed(
            CLIENT_FILES_TABLE
                .as_ref()
                .unwrap()
                .string(file_id as _)
                .ok_or_else(|| format!("Failed to find filepath by file_id: {}", file_id))?
                .to_str()?,
        ) {
            return Ok(None);
        }

        Ok(Some(PackUwUs::build_lua_download_packet(
            file_id,
            PACKUWUS
                .as_ref()
                .unwrap()
                .packed_contents
                .as_ref()
                .ok_or("You forgot to set pack content using PackUwUs_SetPackContent function!")?
                .as_str(),
        )?))
    }

    unsafe {
        if (data as *const u8).read_unaligned() == 4 {
            // 0x00 (sz: 1)    GarrysMod::NetworkMessage::LuaFileDownload aka 4
            // 0x01 (sz: 2)    file number
            // 0x03 (sz: 0x20) file content hash
            // 0x23 (sz: *)    LZMA file content

            let file_id = (data as *const u16).byte_offset(0x01).read_unaligned();

            match build_download_packet(file_id) {
                Ok(packet) => {
                    if let Some(packet) = packet {
                        #[cfg(debug_assertions)]
                        println!(
                            "[PackUwUs] Sending client {} packed file {}",
                            client_id, file_id
                        );

                        return CVENGINESERVER_GMOD_SENDTOCLIENT.call(
                            this,
                            client_id,
                            packet.as_ptr() as _,
                            (packet.len() * 8) as _,
                        );
                    }

                    #[cfg(debug_assertions)]
                    println!(
                        "[PackUwUs] File {} is not packed, sending original content to client {}",
                        file_id, client_id
                    );
                }
                Err(err) => {
                    println!(
                        "[PackUwUs] Error occured while building download packet: {}",
                        err
                    )
                }
            }
        }
    }

    unsafe { CVENGINESERVER_GMOD_SENDTOCLIENT.call(this, client_id, data, data_len) }
}

pub(crate) fn new_cvengineserver_gmod_sendtoclients(
    this: *const c_void,
    filter: *const c_void,
    data: *const c_void,
    data_len: i32,
) {
    #[cfg(debug_assertions)]
    println!(
        "CVEngineServer::GMOD_SendToClient (all clients)({:?}, {:?}, {:?}, {})",
        this, filter, data, data_len
    );

    unsafe fn try_get_new_lua_code(
        filepath: &CStr,
        compressed_lzma_code: &[u8],
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filepath = filepath.to_str()?;

        let original_lua_code = CString::from_vec_with_nul(
            gmod_lzma::decompress(compressed_lzma_code).or_else(|lzma_errnum| {
                Err(format!(
                    "Failed to decompress: status code is {}",
                    lzma_errnum
                ))
            })?,
        )?;

        let (should_pack, new_code) = PACKUWUS
            .as_ref()
            .unwrap()
            .handle_pack(filepath, &original_lua_code.to_string_lossy())?;

        let has_new_code = new_code.is_some();

        let code_to_save =
            new_code.unwrap_or_else(|| original_lua_code.to_string_lossy().to_string());

        if let Err(err) = PACKUWUS
            .as_mut()
            .unwrap()
            .edit_file(filepath, code_to_save.clone())
        {
            println!("[PackUwUs] packUwUs edit file failed: {}", err);
        }

        if should_pack && has_new_code {
            return Ok(Some(code_to_save));
        }

        Ok(None)
    }

    unsafe {
        let data = data as *const u8;

        if data.read_unaligned() == 1 {
            // 0x00      (sz: 1)    ??? but 1
            // 0x01      (sz: \0)   filepath
            // 0x??+0x01 (sz: 4)    compressed size
            // 0x??+0x05 (sz: 0x20) hash
            // 0x??+0x25 (sz: *)    LZMA file content

            //hexdump(slice::from_raw_parts(data, (data_len / 8) as _));

            let filepath = CStr::from_ptr(data.byte_offset(1) as _);

            let compressed_lzma_code = data
                .byte_offset(0x01)
                .byte_offset(filepath.to_bytes_with_nul().len() as _)
                .byte_offset(4)
                .byte_offset(0x20);

            let compressed_lzma_code = slice::from_raw_parts(
                compressed_lzma_code,
                (((data_len / 8) as usize) - 1 - filepath.to_bytes_with_nul().len()) as _,
            );

            match try_get_new_lua_code(filepath, compressed_lzma_code) {
                Ok(new_lua_code) => {
                    if let Some(new_lua_code) = new_lua_code {
                        match PackUwUs::build_lua_autorefresh_packet(
                            filepath.to_str().unwrap(),
                            new_lua_code.as_str(),
                        ) {
                            Ok(packet) => {
                                #[cfg(debug_assertions)]
                                println!("[PackUwUs] Auto-refresh {}", filepath.to_string_lossy());

                                return CVENGINESERVER_GMOD_SENDTOCLIENTS.call(
                                    this,
                                    filter,
                                    packet.as_ptr() as _,
                                    (packet.len() * 8) as _,
                                );
                            }
                            Err(err) => {
                                println!("[PackUwUs] Failed to build autorefresh packet: {}", err)
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("Error occured in try_get_new_lua_code: {}", err)
                }
            }
        }
    }

    unsafe { CVENGINESERVER_GMOD_SENDTOCLIENTS.call(this, filter, data, data_len) }
}
