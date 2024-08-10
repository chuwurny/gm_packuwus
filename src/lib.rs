#![feature(hasher_prefixfree_extras)]

mod module;
mod packuwus;
mod sdk;

use gmod::{
    gmod13_close, gmod13_open,
    lua::{LuaReference, State, LUA_GLOBALSINDEX},
    lua_function, lua_string,
};
use lazy_static::lazy_static;
use module::Module;
use packuwus::PackUwUs;
use procfs::process::Process;
use retour::static_detour;
use sdk::{
    filesystem::WrappedFileSystem,
    luafile::LuaFile,
    networkstringtable::WrappedNetworkStringTable,
    networkstringtablecontainer::{
        NetworkStringTableContainer, WrappedNetworkStringTableContainer,
    },
};
use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    mem::transmute,
    slice,
    sync::{Arc, Mutex},
    thread::{self},
};

static_detour! {
    static GMODDATAPACK_ADDORUPDATEFILE: unsafe extern "C" fn(*const c_void, *mut LuaFile, bool);
    static GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA: unsafe extern "C" fn(*const *const c_char, *const *const c_char, *const *const c_char) -> c_int;
    static CVENGINESERVER_GMOD_SENDTOCLIENT: unsafe extern "C" fn (*const c_void, i32, *const c_void, i32);
    static CVENGINESERVER_GMOD_SENDTOCLIENTS: unsafe extern "C" fn (*const c_void, *const c_void, *const c_void, i32);
}

static mut PACKUWUS: Option<PackUwUs> = None;
static mut CLIENT_FILES_TABLE: Option<WrappedNetworkStringTable> = None;

#[gmod13_open]
fn gmod13_open(lua: State) -> i32 {
    let this_proc = match Process::myself() {
        Ok(proc) => proc,
        Err(err) => unsafe { lua.error(format!("Failed to get myself process: {}", err)) },
    };

    let server_srv = match Module::from_process(&this_proc, "server_srv.so") {
        Ok(server_srv) => server_srv,
        Err(err) => unsafe { lua.error(format!("Failed to get server_srv.so: {}", err)) },
    };

    let engine_srv = match Module::from_process(&this_proc, "engine_srv.so") {
        Ok(engine_srv) => engine_srv,
        Err(err) => unsafe { lua.error(format!("Failed to get engine_srv.so: {}", err)) },
    };

    let network_string_table_container = unsafe {
        match engine_srv.interface::<NetworkStringTableContainer>("VEngineServerStringTable001") {
            Ok(ptr) => Some(WrappedNetworkStringTableContainer(ptr)),
            Err(err) => lua.error(format!(
                "Failed to get VEngineServerStringTable001: {}",
                err
            )),
        }
    };

    let client_lua_files = unsafe {
        match network_string_table_container
            .as_ref()
            .unwrap()
            .find_table("client_lua_files")
            .unwrap()
        {
            Some(tbl) => tbl,
            None => lua.error("Failed to find \"client_lua_files\" network string table"),
        }
    };

    unsafe {
        CLIENT_FILES_TABLE = Some(client_lua_files);
    }

    let downloadables = match network_string_table_container
        .as_ref()
        .unwrap()
        .find_table("downloadables")
        .unwrap()
    {
        Some(tbl) => tbl,
        None => unsafe { lua.error("Failed to find \"downloadables\" network string table") },
    };

    let fs = unsafe {
        match server_srv.symbol("g_pFullFileSystem") {
            Ok(sym) => WrappedFileSystem(*(sym as *const *const _)),
            Err(err) => lua.error(format!("Failed to get g_pFullFileSystem: {}", err)),
        }
    };

    let gmoddatapack_addorupdatefile =
        match server_srv.symbol("_ZN12GModDataPack15AddOrUpdateFileEP7LuaFileb") {
            Ok(sym) => sym,
            Err(err) => unsafe {
                lua.error(format!(
                    "Failed to get GModDataPack::AddOrUpdateFile: {}",
                    err
                ))
            },
        };

    unsafe {
        if let Err(err) = GMODDATAPACK_ADDORUPDATEFILE.initialize(
            transmute(gmoddatapack_addorupdatefile as *const c_void),
            new_gmoddatapack_addorupdatefile,
        ) {
            lua.error(format!(
                "Failed to initialize GModDataPack::AddOrUpdateFile hook: {}",
                err
            ))
        }

        if let Err(err) = GMODDATAPACK_ADDORUPDATEFILE.enable() {
            lua.error(format!(
                "Failed to enable GModDataPack::AddOrUpdateFile hook: {}",
                err
            ))
        }
    }

    let garrysmod_autorefresh_handlechange_lua =
        match server_srv.symbol("_ZN9GarrysMod11AutoRefresh16HandleChange_LuaERKSsS2_S2_") {
            Ok(sym) => sym,
            Err(err) => unsafe {
                lua.error(format!(
                    "Failed to get GarrysMod::AutoRefresh::HandleChange_Lua: {}",
                    err
                ))
            },
        };

    unsafe {
        if let Err(err) = GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA.initialize(
            transmute(garrysmod_autorefresh_handlechange_lua as *const c_void),
            new_garrysmod_autorefresh_handlechange_lua,
        ) {
            lua.error(format!(
                "Failed to initialize GarrysMod::AutoRefresh::HandleChange_Lua hook: {}",
                err
            ))
        }

        if let Err(err) = GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA.enable() {
            lua.error(format!(
                "Failed to enable GarrysMod::AutoRefresh::HandleChange_Lua hook: {}",
                err
            ))
        }
    }

    let cvengineserver_gmod_sendtoclients =
        match engine_srv.symbol("_ZN14CVEngineServer17GMOD_SendToClientEP16IRecipientFilterPvi") {
            Ok(sym) => sym,
            Err(err) => unsafe {
                lua.error(format!(
                    "Failed to get CVEngineServer::GMOD_SendToClient (all clients): {}",
                    err
                ))
            },
        };

    unsafe {
        if let Err(err) = CVENGINESERVER_GMOD_SENDTOCLIENTS.initialize(
            transmute(cvengineserver_gmod_sendtoclients as *const c_void),
            new_cvengineserver_gmod_sendtoclients,
        ) {
            lua.error(format!(
                "Failed to initialize CVEngineServer::GMOD_SendToClient (all clients) hook: {}",
                err
            ))
        }

        if let Err(err) = CVENGINESERVER_GMOD_SENDTOCLIENTS.enable() {
            lua.error(format!(
                "Failed to enable CVEngineServer::GMOD_SendToClient (all clients) hook: {}",
                err
            ))
        }
    }

    let cvengineserver_gmod_sendtoclient =
        match engine_srv.symbol("_ZN14CVEngineServer17GMOD_SendToClientEiPvi") {
            Ok(sym) => sym,
            Err(err) => unsafe {
                lua.error(format!(
                    "Failed to get CVEngineServer::GMOD_SendToClient: {}",
                    err
                ))
            },
        };

    unsafe {
        if let Err(err) = CVENGINESERVER_GMOD_SENDTOCLIENT.initialize(
            transmute(cvengineserver_gmod_sendtoclient as *const c_void),
            new_cvengineserver_gmod_sendtoclient,
        ) {
            lua.error(format!(
                "Failed to initialize CVEngineServer::GMOD_SendToClient hook: {}",
                err
            ))
        }

        if let Err(err) = CVENGINESERVER_GMOD_SENDTOCLIENT.enable() {
            lua.error(format!(
                "Failed to enable CVEngineServer::GMOD_SendToClient hook: {}",
                err
            ))
        }
    }

    unsafe { PACKUWUS = Some(PackUwUs::new(lua, fs, downloadables, client_lua_files)) }

    unsafe {
        lua.push_function(pack);
        lua.set_field(LUA_GLOBALSINDEX, lua_string!("PackUwUs_Pack"));

        lua.push_function(set_pack_content);
        lua.set_field(LUA_GLOBALSINDEX, lua_string!("PackUwUs_SetPackContent"));
    }

    0
}

#[allow(unused_variables)]
#[gmod13_close]
fn gmod13_close(lua: State) -> i32 {
    if let Err(err) = unsafe { GMODDATAPACK_ADDORUPDATEFILE.disable() } {
        println!("[PackUwUs] Failed to disable GModDataPack::AddOrUpdateFile: {}", err);
    }

    if let Err(err) = unsafe { GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA.disable() } {
        println!(
            "[PackUwUs] Failed to disable GarrysMod::AutoRefresh::HandleChange_Lua: {}",
            err
        );
    }

    if let Err(err) = unsafe { CVENGINESERVER_GMOD_SENDTOCLIENT.disable() } {
        println!(
            "[PackUwUs] Failed to disable CVEngineServer::Gmod_SendToClient: {}",
            err
        );
    }

    if let Err(err) = unsafe { CVENGINESERVER_GMOD_SENDTOCLIENTS.disable() } {
        println!(
            "[PackUwUs] Failed to disable CVEngineServer::Gmod_SendToClient (all clients): {}",
            err
        );
    }

    // TODO: there's no module unload support, so these lines are useless i guess?
    //                                            ~~~~~~~~~~~
    //                                                 |
    // \/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/\/
    unsafe {
        lua.push_nil();
        lua.set_field(LUA_GLOBALSINDEX, b"PackUwUs_Pack\0".as_ptr() as _);

        lua.push_nil();
        lua.set_field(LUA_GLOBALSINDEX, b"PackUwUs_SetPackContent\0".as_ptr() as _);
    }

    0
}

#[derive(PartialEq)]
enum ServeFileStatus {
    Idle,
    Working,
    Failed((LuaReference, String)),
    Done((LuaReference, String)),
}

lazy_static! {
    static ref SERVE_FILE_STATUS: Arc<Mutex<ServeFileStatus>> =
        Arc::new(Mutex::new(ServeFileStatus::Idle));
}

const LUA_SYNC_THREAD_TIMER_NAME: &str = "PackUwUs lua sync thread";

unsafe fn start_sync_thread(lua: State) {
    println!("[PackUwUs] Starting lua sync thread...");

    lua.get_global(lua_string!("timer"));

    if lua.is_table(-1) {
        lua.get_field(-1, lua_string!("Create"));

        if lua.is_function(-1) {
            lua.push_string(LUA_SYNC_THREAD_TIMER_NAME); // name
            lua.push_number(0.0); // interval
            lua.push_number(0.0); // reps
            lua.push_function(lua_sync_thread); // callback

            if !lua.pcall_ignore(4, 0) {
                lua.error(format!(
                    "Failed to setup lua sync thread: error occured in _G.timer.Create!"
                ));
            }
        } else {
            lua.pop(); // pop _G.timer.Create

            lua.error(format!(
                "Failed to setup lua sync thread: _G.timer.Create is not a function (got type {})!",
                lua.get_type(-1)
            ));
        }
    } else {
        lua.error(format!(
            "Failed to setup lua sync thread: _G.timer is not a table (got type {})!",
            lua.get_type(-1)
        ));
    }

    lua.pop(); // pop _G.timer
}

unsafe fn stop_sync_thread(lua: State) {
    println!("[PackUwUs] Stopping lua sync thread...");

    lua.get_global(lua_string!("timer"));

    if lua.is_table(-1) {
        lua.get_field(-1, lua_string!("Remove"));

        if lua.is_function(-1) {
            lua.push_string(LUA_SYNC_THREAD_TIMER_NAME); // name

            if !lua.pcall_ignore(1, 0) {
                lua.error(format!(
                    "Failed to stop lua sync thread: error occured in _G.timer.Remove!"
                ));
            }
        } else {
            lua.pop(); // pop _G.timer.Create

            lua.error(format!(
                "Failed to stop lua sync thread: _G.timer.Remove is not a function (got type {})!",
                lua.get_type(-1)
            ));
        }
    } else {
        lua.error(format!(
            "Failed to stop lua sync thread: _G.timer is not a table (got type {})!",
            lua.get_type(-1)
        ));
    }

    lua.pop(); // pop _G.timer
}

#[lua_function]
unsafe fn pack(lua: State) -> i32 {
    match SERVE_FILE_STATUS.try_lock() {
        Ok(ref mut status) => match **status {
            ServeFileStatus::Working => {
                lua.push_boolean(false);

                return 1;
            }
            ServeFileStatus::Failed((callback_ref, _)) => {
                lua.dereference(callback_ref);
            }
            ServeFileStatus::Done((callback_ref, _)) => {
                lua.dereference(callback_ref);
            }
            _ => (),
        },
        Err(_) => {
            lua.push_boolean(false);

            return 1;
        }
    }

    if !PACKUWUS.as_ref().unwrap().content_changed {
        // nothing to repack

        lua.push_boolean(false);

        return 1;
    }

    lua.check_function(1);

    start_sync_thread(lua);

    let callback_ref = lua.reference();

    *SERVE_FILE_STATUS.lock().unwrap() = ServeFileStatus::Working;

    thread::spawn(move || match PACKUWUS.as_mut().unwrap().try_serve() {
        Ok(packed_hash) => {
            if let Some(packed_hash) = packed_hash {
                *SERVE_FILE_STATUS.lock().unwrap() =
                    ServeFileStatus::Done((callback_ref, packed_hash));

                return;
            }
        }
        Err(err) => {
            *SERVE_FILE_STATUS.lock().unwrap() =
                ServeFileStatus::Failed((callback_ref, err.to_string()));
        }
    });

    lua.push_boolean(true);

    1
}

#[lua_function]
unsafe fn lua_sync_thread(lua: State) -> i32 {
    #[cfg(debug_assertions)]
    println!("lua_sync_thread");

    match SERVE_FILE_STATUS.try_lock() {
        Ok(ref mut status) => {
            match **status {
                ServeFileStatus::Failed((callback_ref, ref err)) => {
                    println!("[PackUwUs] Serve file failed: {}", err);

                    lua.from_reference(callback_ref);

                    lua.push_string(err.as_str());
                    lua.push_nil();

                    if !lua.pcall_ignore(2, 0) {
                        println!("[PackUwUs] Error in lua sync thread: PackUwUs_Pack callback errored!");
                    }

                    lua.dereference(callback_ref);

                    **status = ServeFileStatus::Idle;

                    stop_sync_thread(lua);
                }
                ServeFileStatus::Done((callback_ref, ref hash)) => {
                    println!("[PackUwUs] Serve file done! Hash: {}", hash);

                    lua.from_reference(callback_ref);

                    lua.push_nil();
                    lua.push_string(hash.as_str());

                    if !lua.pcall_ignore(2, 0) {
                        println!("[PackUwUs] Error in lua sync thread: PackUwUs_Pack callback errored!");
                    }

                    lua.dereference(callback_ref);

                    **status = ServeFileStatus::Idle;

                    stop_sync_thread(lua);
                }
                _ => (),
            }
        }
        Err(_) => (),
    }

    0
}

#[lua_function]
unsafe fn set_pack_content(lua: State) -> i32 {
    println!("[PackUwUs] Setting pack content");

    PACKUWUS.as_mut().unwrap().packed_contents = Some(lua.check_string(1).to_string());

    0
}

fn new_gmoddatapack_addorupdatefile(this: *const c_void, file: *mut LuaFile, reload: bool) {
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

fn new_garrysmod_autorefresh_handlechange_lua(
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

fn new_cvengineserver_gmod_sendtoclient(
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

fn new_cvengineserver_gmod_sendtoclients(
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
