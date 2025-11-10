--local frame_trees = require("frames-tree")
--local filled_squares = require("filled-squares")
local constants = require("constants")

local max_width = constants.width
local max_height = constants.height
local tile_size = constants.chunk_size

-- Recursively walks the quad tree
-- 1. number nodes with value 1 are added to the table
-- 2. string nodes are also added if they have a correponding tile prototype
-- 3. table nodes are descended into recursively
local function descend_tree(node, tile_positions_table, anchor_x, anchor_y, side_size)
    if anchor_x > max_width or anchor_y > max_height then
        return
    end
    if type(node) == "number" then
        if node == 1 then
            table.insert(tile_positions_table, { anchor_x, anchor_y, side_size })
        end
        return
    end
    if type(node) == "string" then
        if prototypes.equipment['bad-apple-tile-' .. node] then
            table.insert(tile_positions_table, { anchor_x, anchor_y, node })
        end
        return
    end
    -- only leaves table
    -- no math.floor because the side_size is a power of 2
    local new_side_size = side_size / 2
    for n, child in pairs(node) do
        local i = n - 1
        local x = i % 2
        local y = math.floor(i / 2)
        local new_anchor_x = anchor_x + x * new_side_size
        local new_anchor_y = anchor_y + y * new_side_size
        descend_tree(child, tile_positions_table, new_anchor_x, new_anchor_y, new_side_size)
    end
end

local tile_positions = {}

-- Preprocess the graph by walking it and extracting all the tile positions that will be used
local function populate_tile_positions_table(nodetree)
    for _, frametree in pairs(nodetree) do
        local frame_table = {}
        descend_tree(frametree, frame_table, 0, 0, 512)
        table.insert(tile_positions, frame_table)
    end
end

local frames_tree = require("generated.frames-tree")

script.on_init(function()
    log("Populating the table of filled squares")
    populate_tile_positions_table(frames_tree)
end)

script.on_event(defines.events.on_player_placed_equipment, function(eventdata)
    if eventdata.equipment.name == "bad-apple-starter" then
        log("Registering event handling")
        local grid = eventdata.grid
        local framenum = 1

        local function increment_frame()
            --log("frame " .. framenum)
            if framenum > constants.frame_count then
                log("Animation end")
                grid.clear()
                script.on_nth_tick(2, nil)
                return
            end

            grid.clear()

            local current_frame_tree = tile_positions[framenum]

            for _, squareinfo in pairs(current_frame_tree) do
                grid.put { name = "bad-apple-tile-" .. (squareinfo[3]), position = { squareinfo[1], squareinfo[2] } }
            end

            if prototypes.equipment["bad-apple-wholeframe-" .. framenum] ~= nil then
                grid.put { name = "bad-apple-wholeframe-" .. framenum, position = { 0, 0 } }
            end

            framenum = framenum + 1
        end

        increment_frame()

        script.on_nth_tick(2, function(tickeventdata)
            increment_frame()
        end)
    end
end)
