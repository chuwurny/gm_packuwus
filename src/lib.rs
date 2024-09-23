#![feature(hasher_prefixfree_extras)]

mod detours;
mod lua_functions;
mod module;
mod packuwus;
mod sdk;

use detours::{
    new_cvengineserver_gmod_sendtoclient, new_cvengineserver_gmod_sendtoclients,
    new_garrysmod_autorefresh_handlechange_lua, new_gmoddatapack_addorupdatefile,
    CVENGINESERVER_GMOD_SENDTOCLIENT, CVENGINESERVER_GMOD_SENDTOCLIENTS,
    GARRYSMOD_AUTOREFRESH_HANDLECHANGE_LUA, GMODDATAPACK_ADDORUPDATEFILE,
};
use gmod::{
    gmod13_close, gmod13_open,
    lua::{State, LUA_GLOBALSINDEX},
    lua_string,
};
use lua_functions::{pack_async, pack_sync, set_pack_content};
use module::Module;
use packuwus::PackUwUs;
use procfs::process::Process;
use sdk::{
    filesystem::WrappedFileSystem,
    networkstringtable::WrappedNetworkStringTable,
    networkstringtablecontainer::{
        NetworkStringTableContainer, WrappedNetworkStringTableContainer,
    },
};
use std::{ffi::c_void, mem::transmute};

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
        lua.push_function(pack_sync);
        lua.set_field(LUA_GLOBALSINDEX, lua_string!("PackUwUs_PackSync"));

        lua.push_function(pack_async);
        lua.set_field(LUA_GLOBALSINDEX, lua_string!("PackUwUs_PackAsync"));

        lua.push_function(set_pack_content);
        lua.set_field(LUA_GLOBALSINDEX, lua_string!("PackUwUs_SetPackContent"));
    }

    0
}

#[allow(unused_variables)]
#[gmod13_close]
fn gmod13_close(lua: State) -> i32 {
    if let Err(err) = unsafe { GMODDATAPACK_ADDORUPDATEFILE.disable() } {
        println!(
            "[PackUwUs] Failed to disable GModDataPack::AddOrUpdateFile: {}",
            err
        );
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
