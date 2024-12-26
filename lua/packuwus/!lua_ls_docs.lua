-----------------------------------------------------------------------------
--- DO NOT include this file! This file is only for Lua LS documentation! ---
-----------------------------------------------------------------------------
---@diagnostic disable: unused-local, missing-return

---Packs lua files synchronously. Returns SHA-256 hash that matches .bsp file
---name. Returns false if pack is failed. May throw error if something's gone
---really wrong!
---@return string|false
function PackUwUs_PackSync() end

---Packs lua files asynchronously.
---
---Returns false if:
---1. Callback function from previous call is about to be called.
---2. Already packing asynchronously.
---3. Nothing to repack.
---@param callback fun(err: string?, hash: string?)
---@return boolean
function PackUwUs_PackAsync(callback) end

---Sets lua file contents. Usually you may want to set
---```lua
---return myUnpackFunc()()
---```
---to unpack current included packed file
---@param contents string Lua file contents
---@return string|false
function PackUwUs_SetPackContent(contents) end

---Callback function that must be set in `_G`. Function handles lua file
---content packing into .bsp file. You can modify script here however you like!
---
---If you return false then file won't be packed
---
---@param path string Full path to lua. You may want to fix path using
---`PackUwUs.FixPath`
---@param content string Lua file content
---@return string|false
function PackUwUs_HandlePack(path, content) end

