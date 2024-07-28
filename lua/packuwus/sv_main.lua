PackUwUs.NeedToRepack = false
PackUwUs.Ready = false

local files = PackUwUs.Files

local log = PackUwUs.Log
local dbg = PackUwUs.Debug
local warn = PackUwUs.Warn
local ok = PackUwUs.Ok
local err = PackUwUs.Error

function PackUwUs.MarkToRepack()
    if not PackUwUs.Ready then return end
    if PackUwUs.NeedToRepack then return end

    dbg("Marked to repack!")

    PackUwUs.NeedToRepack = true

    timer.Create("packuwus autorepack", 0, 1, PackUwUs.Pack)
end

function PackUwUs.AddFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    if files[fixedPath] then
        warn("File %s (%s) already added, ignoring!", path, fixedPath)

        return
    end

    dbg("Adding file %s (%s)", path, fixedPath)

    files[fixedPath] = path

    PackUwUs.MarkToRepack()
end

function PackUwUs.RemoveFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    if files[fixedPath] then
        files[fixedPath] = nil

        dbg("Removed file %s (%s)", path, fixedPath)

        return true
    end

    log("Failed to remove file %s (%s): not in the file list!", path, fixedPath)

    return false
end

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
                subpath ~= "sh_utils.lua"
        end
    end

    if string.match(path, "^%w+/gamemode/cl_init.lua$") then
        return false
    end

    return true
end

function PackUwUs.AddSendTxt(filepath, path)
    dbg("Adding files from send.txt: %s (%s)", filepath, path)

    local f = file.Open(filepath, "r", path)

    if not f then
        log("Failed to open send.txt by path %s (%s)", filepath, path)

        return false
    end

    local content = f:Read()
    f:Close()

    for _, p in ipairs(PackUwUs.Lines(content)) do
        if string.sub(p, 1, 1) ~= "#" and PackUwUs.ShouldPack(p) then
            PackUwUs.AddFile(p)
        end
    end

    return true
end

function PackUwUs.Pack()
    if PackUwUs.NeedToRepack then
        warn("NeedToRepack is set to false, but we're packing?")
    end

    PackUwUs.NeedToRepack = false
    timer.Remove("packuwus autorepack")

    local packFileHandle = file.Open(PackUwUs.PACKED_TEMP_PATH, "wb", "DATA")

    if not packFileHandle then
        err("Failed to pack %d files: failed to open \"%s\"", #files, PackUwUs.PACKED_PATH)

        return false
    end

    local startTime = SysTime()

    for fixedPath, path in pairs(files) do
        local luaFileHandle = file.Open(fixedPath, "rb", "LUA")

        if luaFileHandle then
            local content = luaFileHandle:Read()
            luaFileHandle:Close()

            local lenBeforeTrim = #content
            content = PackUwUs.TrimCode(content)
            dbg("Trimmed code: file size changed from %d to %d", lenBeforeTrim, #content)

            content = util.Compress(content)

            dbg("Writing %s (%s) to pack (len: %d)", path, fixedPath, #content)

            packFileHandle:Write(path)
            packFileHandle:Write("\0")
            packFileHandle:WriteULong(#content)
            packFileHandle:Write(content)
        else
            warn("Failed to add %s (%s) to pack: cannot open file", path, fixedPath)
        end
    end

    packFileHandle:Close()

    ok("Created packed file \"%s\" in %.4f seconds", PackUwUs.PACKED_TEMP_PATH, SysTime() - startTime)

    local serveSucceed, hash = pcall(PackUwUs_ServeFile, PackUwUs.PACKED_TEMP_PATH)

    if not serveSucceed then
        err("Failed to serve packed file: %s", hash)

        return false
    end

    dbg("Serving packed file with hash %s", hash)

    PackUwUs.packuwus_packed_path:SetString(hash)

    return true
end

function PackUwUs.DumpFileList()
    local f = file.Open("packuwus/filelist.txt", "w", "DATA")

    if not f then
        err("Failed to dump file list: cannot open filelist.txt")

        return false
    end

    local count = 0

    for fixedPath, _ in pairs(files) do
        f:Write(fixedPath)
        f:Write("\n")

        count = count + 1
    end

    dbg("Dumped file list (%d files) into filelist.txt", count)

    return true
end
