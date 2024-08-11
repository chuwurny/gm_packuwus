if not PackUwUs.Ready then
    require("hook")

    hook.Add("InitPostEntity", "packuwus init", function()
        hook.Remove("InitPostEntity", "packuwus init")

        PackUwUs.Ready = true

        PackUwUs_SetPackContent("return unpackMeUwU()()")
        PackUwUs.PackSync()

        timer.Create("PackUwUs auto repack", 1, 0, function()
            PackUwUs.PackAsync(true)
        end)
    end)

    PackUwUs.Log("Loading internal module...")
    require("packuwus")
    PackUwUs.Ok("Internal module loaded!")
else
    PackUwUs.PackAsync()
end
