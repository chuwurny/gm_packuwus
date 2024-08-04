if not ({
        ["all"] = true,
        ["nosounds"] = true,
        ["mapsonly"] = true,
        ["noworkshop"] = true,
    })[GetConVar("cl_downloadfilter"):GetString()]
then
    return PackUwUs.FatalError(PackUwUs.Lang({
        en = "Enter    cl_downloadfilter all    in console to correctly load on the server!",
        ru = "Введи    cl_downloadfilter all    в консоль чтобы загрузиться на сервер!"
    }))
end

if not PackUwUs.Unpack() then
    return PackUwUs.FatalError(PackUwUs.Lang({
        en = "Failed to unpack lua files!",
        ru = "Не удалось распаковать луа файлы!"
    }))
end
