local dbg = PackUwUs.Debug

function PackUwUs_ShouldPack(path, reload)
    local fixedPath = PackUwUs.FixPath(path)
    local shouldPack = not reload and PackUwUs.ShouldPack(fixedPath)

    dbg("PackUwUs_ShouldPack(\"%s\", %s) = %s", path, reload, shouldPack)

    return shouldPack
end

function PackUwUs_ClientFile(path, reload)
    local fixedPath = PackUwUs.FixPath(path)

    dbg("PackUwUs_ClientFile(\"%s\", %s)", path, reload)

    if PackUwUs.ShouldPack(fixedPath) then
        if reload then
            PackUwUs.MarkToRepack()
        else
            PackUwUs.AddFile(fixedPath)
        end
    end
end

function PackUwUs_ModifyContent(path, code)
    dbg("PackUwUs_ModifyContent(\"%s\", #code=%d)", path, #code)

    return "return unpackMeUwU()()"
end

require("hook")

hook.Add("InitPostEntity", "packuwus init", function()
    hook.Remove("InitPostEntity", "packuwus init")

    PackUwUs.Ready = true

    PackUwUs.DumpFileList()
    PackUwUs.Pack()
end)

PackUwUs.AddSendTxt("lua/send.txt", "GAME")

require("packuwus")
