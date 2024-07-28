local files = PackUwUs.Files

local log = PackUwUs.Log
local dbg = PackUwUs.Debug
local ok = PackUwUs.Ok
local warn = PackUwUs.Warn
local err = PackUwUs.Error

function PackUwUs.GetPackedFilePath()
    local filename = "download/data/serve_packuwus/" .. PackUwUs.packuwus_packed_path:GetString() .. ".bsp"

    assert(file.Exists(filename, "GAME"), "Packed file path doesn't exist!")

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

function PackUwUs.HealthCheck()
    local function disconnect(msg)
        PackUwUs_HealthCheck_Message = msg

        err("FATAL ERROR: health check failed: %s", msg)

        gui.OpenURL("http://" .. string.rep(" ", 70) .. msg)

        RunConsoleCommand("disconnect")

        return false
    end

    local lang = GetConVar("gmod_language"):GetString()

    if not ({
            ["all"] = true,
            ["nosounds"] = true,
            ["mapsonly"] = true,
            ["noworkshop"] = true,
        })[GetConVar("cl_downloadfilter"):GetString()]
    then
        local msg

        if lang == "ru" then
            msg = "Введи    cl_downloadfilter all    в консоль чтобы загрузиться на сервер!"
        else -- fallback to english
            msg = "Enter    cl_downloadfilter all    in console to correctly load on the server!"
        end

        return disconnect(msg)
    end

    if not file.Exists(PackUwUs.GetPackedFilePath(), "GAME") then
        local msg

        if lang == "ru" then
            msg = "Не удалось скачать запакованные луа файлы!"
        else -- fallback to english
            msg = "Failed to download packed lua files!"
        end

        return disconnect(msg)
    end

    return true
end
