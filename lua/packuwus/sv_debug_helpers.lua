concommand.Add("packuwus_repack", function(ply)
    if IsValid(ply) then return end

    PackUwUs.Pack()
end)
