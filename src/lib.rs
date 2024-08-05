mod module;
mod packuwus;
mod sdk;

use gmod::{
    gmod13_close, gmod13_open,
    lua::{State, LUA_GLOBALSINDEX},
    lua_function,
};
use hexdump::hexdump;
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
    ptr::copy_nonoverlapping,
    slice,
};
use uuid::Uuid;

static_detour! {
    static GMODDATAPACK_ADDORUPDATEFILE: unsafe extern "C" fn(*const c_void, *mut LuaFile, bool);
    static GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA: unsafe extern "C" fn(*const *const c_char, *const *const c_char, *const *const c_char) -> c_int;
    static CVENGINESERVER_GMOD_SENDTOCLIENT: unsafe extern "C" fn (*const c_void, i32, *const c_void, i32);
    static CVENGINESERVER_GMOD_SENDTOCLIENTS: unsafe extern "C" fn (*const c_void, *const c_void, *const c_void, i32);
}

static mut FILESYSTEM: Option<WrappedFileSystem> = None;
static mut NETWORK_STRING_TABLE_CONTAINTER: Option<
    WrappedNetworkStringTableContainer,
> = None;
static mut CLIENT_FILES_TABLE: Option<WrappedNetworkStringTable> = None;
static mut DOWNLOADABLES_TABLE: Option<WrappedNetworkStringTable> = None;
static mut PACKUWUS: Option<PackUwUs> = None;

#[gmod13_open]
fn gmod13_open(lua: State) -> i32 {
    let this_proc = match Process::myself() {
        Ok(proc) => proc,
        Err(err) => unsafe {
            lua.error(format!("Failed to get myself process: {}", err))
        },
    };

    let server_srv = match Module::from_process(&this_proc, "server_srv.so") {
        Ok(server_srv) => server_srv,
        Err(err) => unsafe {
            lua.error(format!("Failed to get server_srv.so: {}", err))
        },
    };

    let engine_srv = match Module::from_process(&this_proc, "engine_srv.so") {
        Ok(engine_srv) => engine_srv,
        Err(err) => unsafe {
            lua.error(format!("Failed to get engine_srv.so: {}", err))
        },
    };

    unsafe {
        NETWORK_STRING_TABLE_CONTAINTER = match engine_srv
            .interface::<NetworkStringTableContainer>(
            "VEngineServerStringTable001",
        ) {
            Ok(ptr) => Some(WrappedNetworkStringTableContainer(ptr)),
            Err(err) => lua.error(format!(
                "Failed to get VEngineServerStringTable001: {}",
                err
            )),
        };

        CLIENT_FILES_TABLE = match NETWORK_STRING_TABLE_CONTAINTER
            .as_ref()
            .unwrap()
            .find_table("client_lua_files")
            .unwrap()
        {
            Some(tbl) => Some(tbl),
            None => lua.error(
                "Failed to find \"client_lua_files\" network string table",
            ),
        };

        DOWNLOADABLES_TABLE = match NETWORK_STRING_TABLE_CONTAINTER
            .as_ref()
            .unwrap()
            .find_table("downloadables")
            .unwrap()
        {
            Some(tbl) => Some(tbl),
            None => lua
                .error("Failed to find \"downloadables\" network string table"),
        };
    }

    unsafe {
        FILESYSTEM = match server_srv.symbol("g_pFullFileSystem") {
            Ok(sym) => Some(WrappedFileSystem(*(sym as *const *const _))),
            Err(err) => {
                lua.error(format!("Failed to get g_pFullFileSystem: {}", err))
            }
        }
    }

    let gmoddatapack_addorupdatefile = match server_srv
        .symbol("_ZN12GModDataPack15AddOrUpdateFileEP7LuaFileb")
    {
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

    let garrysmod_autorefresh_handlechange_lua = match server_srv
        .symbol("_ZN9GarrysMod11AutoRefresh16HandleChange_LuaERKSsS2_S2_")
    {
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

    let cvengineserver_gmod_sendtoclients = match engine_srv
        .symbol("_ZN14CVEngineServer17GMOD_SendToClientEP16IRecipientFilterPvi")
    {
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

    let cvengineserver_gmod_sendtoclient = match engine_srv
        .symbol("_ZN14CVEngineServer17GMOD_SendToClientEiPvi")
    {
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

    unsafe { PACKUWUS = Some(PackUwUs::new(lua)) }

    unsafe {
        lua.push_function(serve_file);
        lua.set_field(LUA_GLOBALSINDEX, b"PackUwUs_ServeFile\0".as_ptr() as _);
    }

    0
}

#[allow(unused_variables)]
#[gmod13_close]
fn gmod13_close(lua: State) -> i32 {
    if let Err(err) = unsafe { GMODDATAPACK_ADDORUPDATEFILE.disable() } {
        println!("Failed to disable GModDataPack::AddOrUpdateFile: {}", err);
    }

    if let Err(err) =
        unsafe { GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA.disable() }
    {
        println!(
            "Failed to disable GarrysMod::AutoRefresh::HandleChange_Lua: {}",
            err
        );
    }

    if let Err(err) = unsafe { CVENGINESERVER_GMOD_SENDTOCLIENT.disable() } {
        println!(
            "Failed to disable CVEngineServer::Gmod_SendToClient: {}",
            err
        );
    }

    if let Err(err) = unsafe { CVENGINESERVER_GMOD_SENDTOCLIENTS.disable() } {
        println!("Failed to disable CVEngineServer::Gmod_SendToClient (all clients): {}", err);
    }

    unsafe {
        lua.push_nil();
        lua.set_field(LUA_GLOBALSINDEX, b"PackUwUs_ServeFile\0".as_ptr() as _);
    }

    0
}

#[lua_function]
unsafe fn serve_file(lua: State) -> i32 {
    let filepath = lua.check_string(1);

    if !FILESYSTEM.as_ref().unwrap().exists(
        CStr::from_ptr(filepath.as_ptr() as _),
        Some(CStr::from_ptr(b"DATA\0".as_ptr() as _)),
    ) {
        lua.error(format!("File \"{}\" doesn't exist in DATA path", filepath))
    }

    let packed_hash = Uuid::new_v4().simple().to_string();
    let new_packed_filename = format!("{}.bsp", packed_hash);
    let new_packed_filepath = format!("serve_packuwus/{}", new_packed_filename);
    let new_packed_filepath_cstr =
        CString::new(new_packed_filepath.as_str()).unwrap();

    #[cfg(debug_assertions)]
    println!("Moving file from {} to {}", filepath, new_packed_filepath);

    if !FILESYSTEM.as_ref().unwrap().rename(
        CStr::from_ptr(filepath.as_ptr() as _),
        &new_packed_filepath_cstr,
        CStr::from_ptr(b"DATA\0".as_ptr() as _),
    ) {
        lua.error(format!(
            "Failed to rename (move) file {} to {}",
            filepath, new_packed_filepath
        ));
    }

    let mut downloadable_filepath = None;

    for index in 0..DOWNLOADABLES_TABLE.as_ref().unwrap().num_strings() {
        let str = DOWNLOADABLES_TABLE.as_ref().unwrap().string(index).unwrap();

        if str.to_string_lossy().starts_with("data/serve_packuwus/") {
            downloadable_filepath = Some(str);

            break;
        }
    }

    let new_full_packed_filepath = format!("data/{}", new_packed_filepath);
    let new_full_packed_filepath_cstr =
        CString::new(new_full_packed_filepath.as_str()).unwrap();

    if let Some(downloadable_filepath) = downloadable_filepath {
        copy_nonoverlapping(
            new_full_packed_filepath_cstr.as_ptr() as _,
            downloadable_filepath.as_ptr() as _,
            new_full_packed_filepath_cstr.as_bytes_with_nul().len(),
        );
    } else {
        let index = DOWNLOADABLES_TABLE
            .as_ref()
            .unwrap()
            .add_string(true, &new_full_packed_filepath_cstr);

        #[cfg(debug_assertions)]
        println!("Added new value to network string table (index: {})", index);
    }

    #[cfg(debug_assertions)]
    println!("Serving {}", new_full_packed_filepath);

    lua.push_string(packed_hash.as_str());

    1
}

fn new_gmoddatapack_addorupdatefile(
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
        if let Err(err) = PACKUWUS.as_ref().unwrap().notify_client_file(
            (*file).name.as_c_str().to_str().unwrap(),
            reload,
        ) {
            println!("Failed to notify client file: {}", err);
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

    unsafe {
        GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA
            .call(directory, filename, file_ext)
    }
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

    /*
    unsafe {
        dbg!(LUASHARED.as_ref().unwrap().cache("lua/includes/init.lua"));
        dbg!(LUASHARED.as_ref().unwrap().cache("includes/init.lua"));
    }
    */

    unsafe fn try_get_new_lua_code(
        file_id: u16,
        compressed_lzma_code: &[u8],
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filepath = CLIENT_FILES_TABLE
            .as_ref()
            .unwrap()
            .string(file_id as _)
            .ok_or_else(|| {
                format!("Failed to find filepath by file_id: {}", file_id)
            })?
            .to_str()?;

        if !PACKUWUS.as_ref().unwrap().should_pack(filepath, false)? {
            return Ok(None);
        }

        let original_lua_code = CString::from_vec_with_nul(
            gmod_lzma::decompress(compressed_lzma_code).or_else(
                |lzma_errnum| {
                    Err(format!(
                        "Failed to decompress: status code is {}",
                        lzma_errnum
                    ))
                },
            )?,
        )?;

        Ok(PACKUWUS
            .as_ref()
            .unwrap()
            .modify_content(filepath, original_lua_code.to_str()?)?)
    }

    unsafe {
        if (data as *const u8).read_unaligned() == 4 {
            // 0x00 (sz: 1)    GarrysMod::NetworkMessage::LuaFileDownload aka 4
            // 0x01 (sz: 2)    file number
            // 0x03 (sz: 0x20) file content hash
            // 0x23 (sz: *)    LZMA file content

            let file_id =
                (data as *const u16).byte_offset(0x01).read_unaligned();

            let lzma_file_content = slice::from_raw_parts(
                (data as *const u8).byte_offset(0x23),
                ((data_len / 8) - 0x23) as _,
            );

            match try_get_new_lua_code(file_id, lzma_file_content) {
                Ok(new_code) => {
                    if let Some(new_lua_code) = new_code {
                        match PackUwUs::build_lua_download_packet(
                            file_id,
                            new_lua_code.as_str(),
                        ) {
                            Ok(packet) => {
                                #[cfg(debug_assertions)]
                                println!("Packing {}", file_id);

                                return CVENGINESERVER_GMOD_SENDTOCLIENT.call(
                                    this,
                                    client_id,
                                    packet.as_ptr() as _,
                                    (packet.len() * 8) as _,
                                );
                            }
                            Err(err) => println!(
                                "Failed to build lua download packet: {}",
                                err
                            ),
                        }
                    }
                }
                Err(err) => {
                    println!("Error occured in try_get_new_lua_code: {}", err)
                }
            }
        }
    }

    unsafe {
        CVENGINESERVER_GMOD_SENDTOCLIENT.call(this, client_id, data, data_len)
    }
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

        if !PACKUWUS.as_ref().unwrap().should_pack(filepath, true)? {
            return Ok(None);
        }

        let original_lua_code = CString::from_vec_with_nul(
            gmod_lzma::decompress(compressed_lzma_code).or_else(
                |lzma_errnum| {
                    Err(format!(
                        "Failed to decompress: status code is {}",
                        lzma_errnum
                    ))
                },
            )?,
        )?;

        Ok(PACKUWUS
            .as_ref()
            .unwrap()
            .modify_content(filepath, original_lua_code.to_str()?)?)
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
                (((data_len / 8) as usize)
                    - 1
                    - filepath.to_bytes_with_nul().len()) as _,
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
                                println!(
                                    "Packing {}",
                                    filepath.to_string_lossy()
                                );

                                return CVENGINESERVER_GMOD_SENDTOCLIENTS.call(
                                    this,
                                    filter,
                                    packet.as_ptr() as _,
                                    (packet.len() * 8) as _,
                                );
                            }
                            Err(err) => println!(
                                "Failed to build autorefresh packet: {}",
                                err
                            ),
                        }
                    }
                }
                Err(err) => {
                    println!("Error occured in try_get_new_lua_code: {}", err)
                }
            }
        }
    }

    unsafe {
        CVENGINESERVER_GMOD_SENDTOCLIENTS.call(this, filter, data, data_len)
    }
}
