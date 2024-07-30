local files = PackUwUs.Files

local log = PackUwUs.Log
local dbg = PackUwUs.Debug
local ok = PackUwUs.Ok
local warn = PackUwUs.Warn
local err = PackUwUs.Error

function PackUwUs.GetPackedFilePath()
    local filename = "download/data/serve_packuwus/" .. PackUwUs.packuwus_packed_path:GetString() .. ".bsp"

    if not file.Exists(filename, "GAME") then
        err("Cannot get packed file path: packed file doesn't exist!")

        return nil
    end

    return filename
end

function PackUwUs.Unpack()
    local function readString(f)
        local s = ""

        while true do
            if f:EndOfFile() then
                return nil
            end

            local c = f:Read(1)

            if c == "\0" then
                break
            end

            s = s .. c
        end

        return s
    end

    log("Unpacking files")

    for k, _ in pairs(files) do
        files[k] = nil
    end

    local packedFilePath = PackUwUs.GetPackedFilePath()

    if not packedFilePath then
        err("Failed to unpack: no packed file!")

        return false
    end

    local f = file.Open(packedFilePath, "rb", "GAME")

    if not f then
        err("Failed to unpack: failed to open \"%s\"", packedFilePath)

        return false
    end

    local filesCount = 0

    while true do
        if f:EndOfFile() then
            break
        end

        local path = readString(f)

        if not path then
            err("Failed to unpack: unexpected EOF while reading path!")

            return false
        end

        if f:EndOfFile() then
            err("Failed to unpack: unexpected EOF while reading size!")

            return false
        end

        local size = f:ReadULong()

        if f:EndOfFile() then
            err("Failed to unpack: unexpected EOF while reading size of %s!", path)

            return false
        end

        local content = f:Read(size)

        if #content ~= size then
            err("Failed to unpack: readed content size of %s differs (%d != %d)!", path, #content, size)

            return false
        end

        content = util.Decompress(content)

        if not content then
            err("Failed to unpack: decompress %s failed!", path)

            return false
        end

        filesCount = filesCount + 1

        path = PackUwUs.FixPath(path)
        files[path] = content

        dbg("Readed %s (len: %d)", path, #content)
    end

    f:Close()

    ok("Finished unpacking %d files", filesCount)

    return true
end

function PackUwUs.LoadFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    dbg("Loading file %s", fixedPath)

    local content = files[fixedPath]

    if not content then
        error("Failed to load file " .. fixedPath .. ": not in file list")
    end

    return CompileString(content, path)
end

function PackUwUs.FatalError(msg)
    err("FATAL ERROR: %s", msg)

    if PackUwUs.FatalFuckUp then
        return -- already fucked up, stop.
    end

    PackUwUs.FatalFuckUp = true

    gui.OpenURL("http://" .. string.rep(" ", 70) .. msg)

    RunConsoleCommand("disconnect")

    function unpackMeUwU()
        ErrorNoHalt(
            "\n\n!!!!!!!!!!!!!!!!!!!!\n\n" ..
            string.rep(" ", 20) .. msg ..
            "\n\n!!!!!!!!!!!!!!!!!!!!\n" ..
            math.random() .. "\n\n\n"
        )

        return function() end
    end

    require("gamemode")
    require("scripted_ents")
    require("weapons")
end
