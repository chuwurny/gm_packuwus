--- { string path = string content }
---@type { [string]: string }
PackUwUs.Files = PackUwUs.Files or {}

local files = PackUwUs.Files

local log = PackUwUs.Log
local dbg = PackUwUs.Debug
local ok = PackUwUs.Ok
local err = PackUwUs.Error

---Checks if file is packed
---@param path string
---@return boolean
function PackUwUs.HasFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    return files[fixedPath] ~= nil
end

---Returns packed file path relative to GAME. If `noFatalError` is `true` then
---function may return `nil`
---@param noFatalError boolean? `true` will not call `PackUwUs.FatalError`
---@return string?
function PackUwUs.GetPackedFilePath(noFatalError)
    local filename = "download/data/serve_packuwus/" .. PackUwUs.packuwus_hash:GetString() .. ".bsp"

    if not file.Exists(filename, "GAME") then
        err("Cannot get packed file path: packed file doesn't exist!")

        local downloadUrl = GetConVar("sv_downloadurl"):GetString()

        if downloadUrl == "" then
            err("Server did not set sv_downloadurl!")

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "Server did not set sv_downloadurl! Contact admins!",
                    ru = "Сервер не установил sv_downloadurl! Обратись к " ..
                         "админу!",
                }))
            end
        end

        if not noFatalError then
            PackUwUs.FatalError(PackUwUs.Lang({
                en = "No packed file " .. filename .. ". Is website " ..
                     downloadUrl .. " accessable from this PC?",
                ru = "Нету запакованного файла " .. filename .. ". Сайт " ..
                     downloadUrl .. " с компьютера доступен?",
            }))
        end

        return nil
    end

    return filename
end

---Unpacks lua files. `PackUwUs.Files` will be updated. Returns `true` if no
---error is occured
---@param noFatalError boolean? `true` will not call `PackUwUs.FatalError`
---@return boolean
function PackUwUs.Unpack(noFatalError)
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

    local packedFilePath = PackUwUs.GetPackedFilePath(noFatalError)

    if not packedFilePath then
        err("Failed to unpack: no packed file!")

        return false
    end

    local f = file.Open(packedFilePath, "rb", "GAME") --[[@as File]]

    if not f then
        err("Failed to unpack: failed to open \"%s\"", packedFilePath)

        if not noFatalError then
            PackUwUs.FatalError(PackUwUs.Lang({
                en = "Failed to open packed file " .. packedFilePath,
                ru = "Неудалось открыть запакованный файл " .. packedFilePath,
            }))
        end

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

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "Failed to read lua file path while unpacking. " ..
                         "Try to delete file " .. packedFilePath,
                    ru = "Неудалось прочитать путь к луа файлу при " ..
                         "распаковке .. Попробуй удалить файл " ..
                         packedFilePath,
                }))
            end

            return false
        end

        if f:EndOfFile() then
            err("Failed to unpack: unexpected EOF while reading size!")

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "No more packed data to read. Packed lua files " ..
                         packedFilePath .. " is corrupted! Delete the file " ..
                         "and try to connect again",
                    ru = "Нет данных для чтения. Запакованные луа файлы " ..
                         packedFilePath .. " были повреждены! Удали файл и " ..
                         "попробуй зайти снова",
                }))
            end

            return false
        end

        local size = f:ReadULong()

        if f:EndOfFile() then
            err("Failed to unpack: unexpected EOF while reading size of %s!", path)

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "No more packed data to read. Packed lua files " ..
                         packedFilePath .. " is corrupted! Delete the file " ..
                         "and try to connect again",
                    ru = "Нет данных для чтения. Запакованные луа файлы " ..
                         packedFilePath .. " были повреждены! Удали файл и " ..
                         "попробуй зайти снова",
                }))
            end

            return false
        end

        local content = f:Read(size)

        if #content ~= size then
            err("Failed to unpack: readed content size of %s differs (%d != %d)!", path, #content, size)

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "Packed lua file " .. path .. " got corrupted! " ..
                         "Delete " .. packedFilePath .. " and try to " ..
                         "connect again",
                    ru = "Запакованный луа файл " .. path .. " повреждён! " ..
                         "Удали " .. packedFilePath .. " и попробуй зайти " ..
                         "снова",
                }))
            end

            return false
        end

        content = util.Decompress(content)

        if not content then
            err("Failed to unpack: decompress %s failed!", path)

            if not noFatalError then
                PackUwUs.FatalError(PackUwUs.Lang({
                    en = "Failed to decompress lua file " .. path .. ". No " ..
                         "free RAM?",
                    ru = "Неудалось распаковать файл " .. path .. ". Нет " ..
                         "свободной ОЗУ?",
                }))
            end

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

---Compiles lua file into function using `CompileString`. If lua file is not
---packed then error will be thrown
---@param path string Path to lua file
---@return fun(...): ...
function PackUwUs.LoadFile(path)
    local fixedPath = PackUwUs.FixPath(path)

    dbg("Loading file %s", fixedPath)

    local content = files[fixedPath]

    if not content then
        err("Failed to load file %s: not in file list", fixedPath)

        error("Failed to load file " .. fixedPath .. ": not in file list")
    end

    return CompileString(content, path)
end

---Disconnects user and show message on the client's screen
---@param msg string Message to show
function PackUwUs.FatalError(msg)
    err("FATAL ERROR: %s", msg)

    if PackUwUs.FatalFuckUp then
        return -- already fucked up, stop.
    end

    PackUwUs.FatalFuckUp = true

    gui.OpenURL("http://" .. string.rep(" ", 10) .. msg)

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

    error("Fatal PackUwUs error: " .. msg)
end
