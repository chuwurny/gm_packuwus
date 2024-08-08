local gsub = string.gsub

local function trimLineSpaces(lines)
    for i = 1, #lines do
        local line = lines[i]

        line = gsub(line, "^%s+", "")
        line = gsub(line, "%s+$", "")

        lines[i] = line
    end
end

local function removeEmptyLinesAtEOF(lines)
    for i = #lines, 1, -1 do
        if lines[i] == "" then
            lines[i] = nil
        else
            break
        end
    end
end

function PackUwUs.TrimCode(code)
    local lines = PackUwUs.Lines(code, true)

    trimLineSpaces(lines)
    removeEmptyLinesAtEOF(lines)

    return table.concat(lines, "\n")
end
