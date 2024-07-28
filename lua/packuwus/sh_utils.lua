local dbg = PackUwUs.Debug

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

function PackUwUs.Basename(path)
    for i = #path, 1, -1 do
        if string.sub(path, i, i) == "/" then
            return string.sub(path, i + 1)
        end
    end

    return path
end

function PackUwUs.Extensionless(filename)
    return string.match(filename, "^(.+)%..*") or filename
end

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
