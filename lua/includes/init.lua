AddCSLuaFile()
include("packuwus/sh_main.lua")

AddCSLuaFile("includes/_init.lua")
include("includes/_init.lua")

if not gamemode then
    return PackUwUs.FatalError(PackUwUs.Lang({
        en = "'_G.gamemode' is nil! Looks like serverside pack error!",
        ru = "'_G.gamemode' это nil! Похоже ошибка при запаковке на стороне сервера!",
    }))
end

if CLIENT then
    include("packuwus/cl_debug_helpers.lua")
else
    include("packuwus/sv_debug_helpers.lua")
end
