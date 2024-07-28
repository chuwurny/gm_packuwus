__PackUwUs__old_CompileFile = CompileFile

local log = PackUwUs.Log

if not PackUwUs.HealthCheck() then
    function unpackMeUwU()
        for _ = 1, 10 do
            ErrorNoHalt(
                "\n\n!!!!!!!!!!!!!!!!!!!!\n\n                 " ..
                PackUwUs_HealthCheck_Message ..
                "\n\n!!!!!!!!!!!!!!!!!!!!\n" ..
                math.random() .. "\n\n\n"
            )
        end

        return function() end
    end

    require("gamemode")
    require("scripted_ents")
    require("weapons")

    return
end

PackUwUs.Unpack()

local function calleeFilePath()
    return assert(string.match(debug.traceback(3), "\n%s+([^:]+):%d+: in main"),
        "Failed to get function path in main chunk")
end

function unpackMeUwU()
    local path = calleeFilePath()

    local fixedPath = PackUwUs.FixPath(path)

    log("Unpacking \"%s\"", fixedPath)

    return PackUwUs.LoadFile(fixedPath)
end

function CompileFile(path)
    if not PackUwUs.HasFile(path) then
        return __PackUwUs__old_CompileFile(path)
    end

    log("Compiling file \"%s\"", path)

    return PackUwUs.LoadFile(path)
end
