use std::{
    sync::{Arc, Mutex},
    thread,
};

use gmod::{
    lua::{LuaReference, State},
    lua_function, lua_string,
};
use lazy_static::lazy_static;

use crate::PACKUWUS;

const LUA_SYNC_THREAD_TIMER_NAME: &str = "PackUwUs lua sync thread";

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
pub(crate) unsafe fn pack_sync(lua: State) -> i32 {
    match PACKUWUS.as_mut().unwrap().try_serve() {
        Ok(hash) => {
            if let Some(hash) = hash {
                lua.push_string(hash.as_str());

                return 1;
            }
        }
        Err(err) => lua.error(format!("Failed to pack: {}", err)),
    };

    lua.push_boolean(false);

    1
}

#[lua_function]
pub(crate) unsafe fn pack_async(lua: State) -> i32 {
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
        Ok(ref mut status) => match **status {
            ServeFileStatus::Failed((callback_ref, ref err)) => {
                println!("[PackUwUs] Serve file failed: {}", err);

                lua.from_reference(callback_ref);

                lua.push_string(err.as_str());
                lua.push_nil();

                if !lua.pcall_ignore(2, 0) {
                    println!(
                        "[PackUwUs] Error in lua sync thread: PackUwUs_Pack callback errored!"
                    );
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
                    println!(
                        "[PackUwUs] Error in lua sync thread: PackUwUs_Pack callback errored!"
                    );
                }

                lua.dereference(callback_ref);

                **status = ServeFileStatus::Idle;

                stop_sync_thread(lua);
            }
            _ => (),
        },
        Err(_) => (),
    }

    0
}

#[lua_function]
pub(crate) unsafe fn set_pack_content(lua: State) -> i32 {
    println!("[PackUwUs] Setting pack content");

    PACKUWUS.as_mut().unwrap().packed_contents = Some(lua.check_string(1).to_string());

    0
}
