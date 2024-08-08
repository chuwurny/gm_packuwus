local dbg = PackUwUs.Debug

function PackUwUs_HandlePack(path, content)
    if not PackUwUs.ShouldPack(path) then
        return false
    end

    dbg("PackUwUs_HandlePack(\"%s\", #%d)", path, #content)

    return PackUwUs.TrimCode(content)
end
