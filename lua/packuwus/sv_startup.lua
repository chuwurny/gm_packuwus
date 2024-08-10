if not PackUwUs.Ready then
    require("hook")

    hook.Add("InitPostEntity", "packuwus init", function()
        hook.Remove("InitPostEntity", "packuwus init")

        PackUwUs.Ready = true

        PackUwUs.PackSync()
        PackUwUs_SetPackContent("return unpackMeUwU()()")

        timer.Create("PackUwUs auto repack", 1, 0, PackUwUs.PackAsync)
    end)

    PackUwUs.Log("Loading internal module...")
    require("packuwus")
    PackUwUs.Ok("Internal module loaded!")
else
    PackUwUs.PackAsync()
end
