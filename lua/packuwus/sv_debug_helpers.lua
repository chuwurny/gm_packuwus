concommand.Add("packuwus_repack_async", function(ply)
    if IsValid(ply) then return end

    PackUwUs.PackAsync()
end)

concommand.Add("packuwus_repack_sync", function(ply)
    if IsValid(ply) then return end

    PackUwUs.PackSync()
end)
