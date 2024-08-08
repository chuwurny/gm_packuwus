if not PackUwUs.Ready then
    require("hook")

    hook.Add("InitPostEntity", "packuwus init", function()
        hook.Remove("InitPostEntity", "packuwus init")

        PackUwUs.Ready = true

        PackUwUs.Pack()
        PackUwUs_SetPackContent("return unpackMeUwU()()")
    end)

    PackUwUs.Log("Loading internal module...")
    require("packuwus")
    PackUwUs.Ok("Internal module loaded!")
else
    PackUwUs.Pack()
end
