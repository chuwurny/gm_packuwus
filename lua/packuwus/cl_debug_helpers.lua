local files = PackUwUs.Files
local log = PackUwUs.Log
local err = PackUwUs.Error

concommand.Add("packuwus_dump_file", function(_, _, _, path)
    if not PackUwUs.HasFile(path) then
        return err("packuwus_dump_file: File \"%s\" not packed!", path)
    end

    local fixedPath = PackUwUs.FixPath(path)

    log("packuwus_dump_file: Dumping \"%s\" to console\n%s", fixedPath, files[fixedPath])
end)
