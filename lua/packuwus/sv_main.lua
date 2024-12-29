PackUwUs.NeedToRepack = PackUwUs.NeedToRepack or false
PackUwUs.Ready        = PackUwUs.Ready or false
PackUwUs.Packing      = PackUwUs.Packing or false

---@type number[]
PackUwUs._ConnectingUserIDs = PackUwUs._ConnectingUserIDs or {}

local connectUserIDs = PackUwUs._ConnectingUserIDs

local log = PackUwUs.Log
local warn = PackUwUs.Warn
local ok = PackUwUs.Ok
local err = PackUwUs.Error
local dbg = PackUwUs.Debug

---Returns `true` if file in this path should be packed
---@param path string
---@return boolean
function PackUwUs.ShouldPack(path)
    path = PackUwUs.FixPath(path)

    if
        path == "includes/init.lua" or
        path == "skins/default.lua"
    then
        return false
    end

    do
        local subpath = string.match(path, "^packuwus/(.+)$")

        if subpath then
            return subpath ~= "sh_main.lua" and
                subpath ~= "cl_main.lua" and
                subpath ~= "cl_impl.lua" and
                subpath ~= "sh_utils.lua" and
                subpath ~= "cl_startup.lua"
        end
    end

    if string.match(path, "^%w+/gamemode/cl_init.lua$") then
        return false
    end

    return true
end

---Safe function to pack lua files synchronously. Internally calls
---`PackUwUs_PackSync`
---@param onlyCheck boolean? If set to true then it prevents setting
---`NeedToRepack` to `true`. You can use it to auto repack lua files
function PackUwUs.PackSync(onlyCheck)
    if PackUwUs.Packing then
        if onlyCheck ~= true then
            PackUwUs.NeedToRepack = true

            dbg("NeedToRepack set to true")
        end

        return
    end

    local startTime = SysTime()

    log("Packing UwUs...")
    dbg("Packing synchronously")

    local success, result = pcall(PackUwUs_PackSync)

    if not success then
        err("Error occured while packing: %s", result)
    elseif result == false then
        ok("Nothing to pack!")
    else
        ok("Pack complete in %.2f seconds! Hash is %s", SysTime() - startTime, result)

        PackUwUs.packuwus_hash:SetString(result)
    end
end

local function addConnectingUserID(userID)
    dbg("Adding connecting user id %d. Now connecting count is %d",
        userID,
        #connectUserIDs)

    table.insert(connectUserIDs, userID)
end

local function tryRemoveConnectingUserID(userIDToRemove)
    for i, userID in ipairs(connectUserIDs) do
        if userID == userIDToRemove then
            dbg("Remove connecting user id %d. Now connecting count is %d",
                userID,
                #connectUserIDs)

            table.remove(connectUserIDs, i)

            break
        end
    end
end

---Safe function to pack lua files asynchronously. Internally calls
---`PackUwUs_PackAsync`
---@param onlyCheck boolean? If set to true then it prevents setting
---`NeedToRepack` to `true`. You can use it to auto repack lua files
function PackUwUs.PackAsync(onlyCheck)
    if PackUwUs.Packing then
        if onlyCheck ~= true then
            PackUwUs.NeedToRepack = true

            dbg("NeedToRepack set to true")
        end

        return
    end

    local startTime = SysTime()

    local packStarted = PackUwUs_PackAsync(function(packErr, hash)
        local lastConnectingCount

        timer.Create("packuwus async pack wait for players", 1, 0, function()
            if #connectUserIDs ~= 0 then
                if lastConnectingCount ~= #connectUserIDs then
                    lastConnectingCount = #connectUserIDs

                    warn("Delaying hash update due to players connecting " ..
                         "(%d)",
                         lastConnectingCount)
                end

                return
            end

            timer.Remove("packuwus async pack wait for players")

            PackUwUs.Packing = false

            if packErr then
                err("Error occured while packing: %s", packErr)
            else
                ok("Pack complete in %.2f seconds! Hash is %s", SysTime() - startTime, hash)

                PackUwUs.packuwus_hash:SetString(hash)
            end

            if PackUwUs.NeedToRepack then
                warn("NeedToRepack is set while packing, repacking...")

                PackUwUs.PackAsync()
            end
        end)
    end)

    if packStarted then
        log("Packing UwUs...")
        dbg("Packing asynchronously")

        PackUwUs.NeedToRepack = false
        PackUwUs.Packing = true
    end
end

gameevent.Listen("player_connect")
hook.Add("player_connect", "packuwus connecting workaround", function(data)
    ---@cast data player_connect

    if data.bot == 1 then return end

    addConnectingUserID(data.userid)
end)

gameevent.Listen("player_disconnect")
hook.Add("player_disconnect", "packuwus connecting workaround", function(data)
    ---@cast data player_disconnect

    if data.bot == 1 then return end

    tryRemoveConnectingUserID(data.userid)
end)

hook.Add("PlayerInitialSpawn", "packuwus connecting workaround", function(ply)
    ---@cast ply Player

    if ply:IsBot() then return end

    tryRemoveConnectingUserID(ply:UserID())
end)

