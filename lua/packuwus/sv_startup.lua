if not PackUwUs.Ready then
    require("hook")

    PackUwUs.AddSendTxt("lua/send.txt", "GAME")

    hook.Add("InitPostEntity", "packuwus init", function()
        hook.Remove("InitPostEntity", "packuwus init")

        PackUwUs.Ready = true

        PackUwUs.DumpFileList()
        PackUwUs.Pack(false)
    end)

    require("packuwus")
else
    PackUwUs.MarkToRepack()
end
