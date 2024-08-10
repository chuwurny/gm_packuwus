PackUwUs.NeedToRepack = PackUwUs.NeedToRepack or false
PackUwUs.Ready        = PackUwUs.Ready or false
PackUwUs.Packing      = PackUwUs.Packing or false

local log = PackUwUs.Log
local warn = PackUwUs.Warn
local ok = PackUwUs.Ok
local err = PackUwUs.Error

function PackUwUs.ShouldPack(path)
    path = PackUwUs.FixPath(path)

    if
        path == "includes/init.lua" or
        path == "skins/default.lua"
    then
        return false
    end

    do
        local subpath = string.match(path, "^packuwus/(.+)$")

        if subpath then
            return subpath ~= "sh_main.lua" and
                subpath ~= "cl_main.lua" and
                subpath ~= "cl_impl.lua" and
                subpath ~= "sh_utils.lua" and
                subpath ~= "cl_startup.lua"
        end
    end

    if string.match(path, "^%w+/gamemode/cl_init.lua$") then
        return false
    end

    return true
end

function PackUwUs.Pack()
    if PackUwUs.Packing then
        PackUwUs.NeedToRepack = true

        return
    end

    local startTime = SysTime()

    local packStarted = PackUwUs_Pack(function(packErr, hash)
        PackUwUs.Packing = false

        if packErr then
            err("Error occured while packing: %s", packErr)
        else
            ok("Pack complete in %.2f seconds! Hash is %s", SysTime() - startTime, hash)

            PackUwUs.packuwus_hash:SetString(hash)
        end

        if PackUwUs.NeedToRepack then
            warn("NeedToRepack is set while packing, repacking...")

            PackUwUs.Pack()
        end
    end)

    if packStarted then
        log("Packing UwUs...")

        PackUwUs.NeedToRepack = false
        PackUwUs.Packing = true
    end
end
