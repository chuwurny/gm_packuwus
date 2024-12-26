local dbg = PackUwUs.Debug

---Returns parent directory
---@see string.GetPathFromFilename
---@return string
function PackUwUs.DirName(path)
    for i = #path, 1, -1 do
        if string.sub(path, i, i) == "/" then
            local dir = string.sub(path, 1, i - 1)

            dbg("DirName(\"%s\") = \"%s\"", path, dir)

            return dir
        end
    end

    dbg("DirName(\"%s\"): failed to find parent directory", path)

    return ""
end

---Returns filename or last directory in path
---@see string.GetFileFromFilename
---@return string
function PackUwUs.Basename(path)
    for i = #path, 1, -1 do
        if string.sub(path, i, i) == "/" then
            return string.sub(path, i + 1)
        end
    end

    return path
end

---Returns filename without extension
---@return string
function PackUwUs.Extensionless(filename)
    return string.match(filename, "^(.+)%..*") or filename
end

---Returns lines from `str`
---@param str string
---@param keepEmpty boolean? If true then saves empty lines
---@return string[]
function PackUwUs.Lines(str, keepEmpty)
    local lines = {}

    if keepEmpty then
        local prevPos = 1

        while true do
            local pos, endPos = string.find(str, "\n", prevPos)

            local line = string.sub(str, prevPos, pos and pos - 1 or nil)

            table.insert(lines, line)

            if pos == nil then
                break
            end

            prevPos = endPos + 1
        end
    else
        for line in string.gmatch(str, "[^\r\n]+") do
            table.insert(lines, line)
        end
    end

    return lines
end

---Returns string based on `gmod_language` convar. If current gmod language is
---is not in `lookupTable`, then "en" will be picked. If there's no "en", then
---it will pick first entry in `lookupTable` using `next` function
---
---Example:
---```lua
---PackUwUs.Lang({
---    en = "English string!",
---    ru = "Русская строка!",
---})
---```
---
---@param lookupTable { [string]: string }
---@return
function PackUwUs.Lang(lookupTable)
    local lang = GetConVar("gmod_language"):GetString()

    return lookupTable[lang] or lookupTable.en or select(2, next(lookupTable))
end
