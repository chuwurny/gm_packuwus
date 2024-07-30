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
    else     -- fallback to english
        msg = "Enter    cl_downloadfilter all    in console to correctly load on the server!"
    end

    return PackUwUs.FatalError(msg)
end

if not PackUwUs.Unpack() then
    local msg

    if lang == "ru" then
        msg = "Не удалось распаковать луа файлы!"
    else -- fallback to english
        msg = "Failed to unpack lua files!"
    end

    return PackUwUs.FatalError(msg)
end
