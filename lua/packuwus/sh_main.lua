AddCSLuaFile()

PackUwUs = PackUwUs or {}
PackUwUs.PACKED_TEMP_PATH = "packuwus/packed.dat"
PackUwUs.packuwus_packed_path = CreateConVar("packuwus_packed_path", "", FCVAR_REPLICATED)

--[[
    SERVER:
    { string fixedPath = string path }

    CLIENT:
    { string path = string content }
]]
PackUwUs.Files = PackUwUs.Files or {}

local files = PackUwUs.Files

file.CreateDir("packuwus")
PackUwUs.LogFileHandle = file.Open("packuwus/log.txt", "w", "DATA")

if not PackUwUs.LogFileHandle then
    print("!!! PackUwUs failed to open \"packuwus/log.txt\" !!!")
end

local logFileHandle = PackUwUs.LogFileHandle
local packuwus_debug = CreateConVar("packuwus_debug", "1", FCVAR_ARCHIVE)
local packuwus_console_debug = CreateConVar("packuwus_console_debug", "0", FCVAR_ARCHIVE)

function PackUwUs.IsDebugEnabled()
    return packuwus_debug:GetBool()
end

local CowoR_CUTE    = { r = 225, g = 150, b = 255, a = 255 }
local COLOR_DEBUG   = { r = 155, g = 230, b = 255, a = 255 }
local COLOR_DEFAULT = { r = 255, g = 255, b = 255, a = 255 }
local COLOR_OK      = { r = 0, g = 255, b = 150, a = 255 }
local COLOR_WARNING = { r = 255, g = 150, b = 0, a = 255 }
local COLOR_ERROR   = { r = 255, g = 150, b = 150, a = 255 }

function PackUwUs.LogEx(level, color, fmt, ...)
    xpcall(function(...)
        if logFileHandle then
            logFileHandle:Write(string.format("%s\t%.5f\t" .. fmt .. "\n", level, SysTime(), ...))

            logFileHandle:Flush()
        end

        if level == "D" and not packuwus_console_debug:GetBool() then
            return
        end

        MsgC(CowoR_CUTE, "[PackUwUs] ", color, string.format(fmt .. "\n", ...))
    end, ErrorNoHaltWithStack, ...)
end

function PackUwUs.Debug(fmt, ...)
    if not PackUwUs.IsDebugEnabled() then return end

    PackUwUs.LogEx("D", COLOR_DEBUG, fmt, ...)
end

function PackUwUs.Log(fmt, ...)
    PackUwUs.LogEx("LOG", COLOR_DEFAULT, fmt, ...)
end

function PackUwUs.Ok(fmt, ...)
    PackUwUs.LogEx("OK", COLOR_OK, fmt, ...)
end

function PackUwUs.Warn(fmt, ...)
    PackUwUs.LogEx("W", COLOR_WARNING, fmt, ...)
end

function PackUwUs.Error(fmt, ...)
    PackUwUs.LogEx("E", COLOR_ERROR, fmt, ...)
end

function PackUwUs.FixPath(path)
    path = string.lower(path)
    path = string.gsub(path, "\\", "/")
    path = string.gsub(path, "/+", "/")

    local parts = {}

    for part in string.gmatch(path, "([^/]+)") do
        if part == ".." then
            table.remove(parts)
        elseif part ~= "." then
            table.insert(parts, part)
        end
    end

    local partPath = table.concat(parts, "/")

    local addonlessPath = string.match(partPath, "^addons/[^/]+/(.+)$")

    if addonlessPath then
        PackUwUs.Debug("FixPath(\"%s\"): removing addon prefix = \"%s\"", path, partPath)

        partPath = addonlessPath
    end

    local prefixlessPath = string.match(partPath, "^lua/(.+)$")

    if prefixlessPath then
        PackUwUs.Debug("FixPath(\"%s\") hit lua = \"%s\"", path, prefixlessPath)

        return prefixlessPath
    end

    prefixlessPath = string.match(partPath, "^gamemodes/[^/]+/entities/(.+)$")

    if prefixlessPath then
        PackUwUs.Debug("FixPath(\"%s\") hit gm ents = \"%s\"", path, prefixlessPath)

        return prefixlessPath
    end

    prefixlessPath = string.match(partPath, "^gamemodes/([^/]+/gamemode/.+)$")

    if prefixlessPath then
        PackUwUs.Debug("FixPath(\"%s\") hit gm gm = \"%s\"", path, prefixlessPath)

        return prefixlessPath
    end

    PackUwUs.Debug("FixPath(\"%s\") did not hit any pattern, returning \"%s\"", path, partPath)

    return partPath
end

function PackUwUs.HasFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    return files[fixedPath] ~= nil
end

AddCSLuaFile("packuwus/sh_utils.lua")
AddCSLuaFile("packuwus/cl_main.lua")
AddCSLuaFile("packuwus/cl_impl.lua")

include("packuwus/sh_utils.lua")

if CLIENT then
    include("packuwus/cl_main.lua")
    include("packuwus/cl_impl.lua")
    --include("packuwus/cl_overrides.lua")
end

if SERVER then
    include("packuwus/sv_utils.lua")
    include("packuwus/sv_main.lua")
    include("packuwus/sv_impl.lua")
end

-- adding them cus we want them pack
AddCSLuaFile("packuwus/cl_debug_helpers.lua")
