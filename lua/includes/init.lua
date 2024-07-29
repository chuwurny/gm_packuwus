AddCSLuaFile()
include("packuwus/sh_main.lua")

AddCSLuaFile("includes/_init.lua")
include("includes/_init.lua")

if CLIENT then
    include("packuwus/cl_debug_helpers.lua")
else
    include("packuwus/sv_debug_helpers.lua")
end
