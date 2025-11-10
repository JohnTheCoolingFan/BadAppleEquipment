local constants = require("constants")
local width = constants.width
local height = constants.height

local chunk_size = constants.chunk_size

local item_sounds = require("__base__.prototypes.item_sounds")
local sounds = require("__base__.prototypes.entity.sounds")

local border_color = constants.border_color
local proto_type = constants.equipment_prototype_type

local frames_tree = require("generated.frames-tree")
local tiles_repeating = require("generated.repeating-tiles")

local equipment_sprite_definition = {
    filename = "__base__/graphics/equipment/fission-reactor-equipment.png",
    width = 1,
    height = 1,
    priority = "medium",
    --scale = 0.005
}
local bad_apple_categories = { "bad-apple" }

data.raw['utility-sprites']['default'].equipment_slot.scale =
    data.raw['utility-sprites']['default'].equipment_slot.scale / constants.grid_cell_scale

-- Decode a hexadecimal encoded string representing a bitmask of pixel values.
-- For every 1 puts the coordinates of the pixel into `points`, using `anchor_x` and `anchor_y` as the top-left corner.
-- Hardcoded to 64 bit for now, could generalize by processing in chunks of 32 (already do but hardcoded)
-- Although tile size 8 already is the max, 360 doesn't divide by 16 cleanly
local function decode_string(encoded_string, points, anchor_x, anchor_y)
    local part1 = string.sub(encoded_string, 1, 8)
    local part0 = string.sub(encoded_string, 9, 16)
    local tilenum0 = tonumber(part0, 16)
    local tilenum1 = tonumber(part1, 16)

    local i = 0

    while tilenum0 ~= 0 do
        if (tilenum0 % 2) == 1 then
            local x = i % chunk_size
            local y = math.floor(i / chunk_size)

            table.insert(points, { anchor_x + x, anchor_y + y })
        end
        i = i + 1
        tilenum0 = math.floor(tilenum0 / 2)
    end

    i = 32

    while tilenum1 ~= 0 do
        if (tilenum1 % 2) == 1 then
            local x = i % chunk_size
            local y = math.floor(i / chunk_size)

            table.insert(points, { anchor_x + x, anchor_y + y })
        end
        i = i + 1
        tilenum1 = math.floor(tilenum1 / 2)
    end
end

-- Recursively walks the quad tree
-- 1. number nodes are ignored (full or empty tiles)
-- 2. string nodes are added to the combined frame unless they are one of the repeating tiles
-- 3. table nodes are descended into recursively
local function descend_tree_combined_frame(node, points, anchor_x, anchor_y, side_size)
    if anchor_x > width or anchor_y > height then
        return
    end
    if type(node) == "number" then
        return
    end
    if type(node) == "string" then
        if data.raw[proto_type]['bad-apple-tile-' .. node] == nil then
            decode_string(node, points, anchor_x, anchor_y)
        end
        return
    end
    -- only leaves table
    local new_side_size = side_size / 2
    for n, child in pairs(node) do
        local i = n - 1
        local x = i % 2
        local y = math.floor(i / 2)
        local new_anchor_x = anchor_x + x * new_side_size
        local new_anchor_y = anchor_y + y * new_side_size
        descend_tree_combined_frame(child, points, new_anchor_x, new_anchor_y, new_side_size)
    end
end

-- Repeating tiles: tiles that appear more than once in the entire animation

local repeating_tiles_prototypes = {}

for _, tile_id in pairs(tiles_repeating) do
    local points = {}
    decode_string(tile_id, points, 0, 0)
    if #points > 0 then
        table.insert(repeating_tiles_prototypes,
            {
                type = proto_type,
                name = "bad-apple-tile-" .. tile_id,
                localised_name = { 'item-name.bad-apple-tile', tile_id },
                localised_description = { 'item-description.bad-apple-tile' },
                hidden_in_factoriopedia = true,
                take_result = "bad-apple-starter",
                background_border_color = border_color,
                sprite = equipment_sprite_definition,
                shape =
                {
                    type = "manual",
                    width = chunk_size,
                    height = chunk_size,
                    points = points
                },
                energy_source =
                {
                    type = "electric",
                    usage_priority = "primary-input"
                },
                energy_consumption = "1W",
                categories = bad_apple_categories
            })
    end
end

data:extend(repeating_tiles_prototypes)

-- Wholeframe: equipment that covers the entire frame, combines the tiles that are used only once

for framenum, node in pairs(frames_tree) do
    local points = {}

    descend_tree_combined_frame(node, points, 0, 0, 512)

    if #points > 0 then
        data:extend({
            {
                type = proto_type,
                name = "bad-apple-wholeframe-" .. framenum,
                localised_name = { 'item-name.bad-apple-tile', tile_id },
                localised_description = { 'item-description.bad-apple-wholeframe' },
                hidden_in_factoriopedia = true,
                take_result = "bad-apple-starter",
                background_border_color = border_color,
                sprite = equipment_sprite_definition,
                shape =
                {
                    type = "manual",
                    width = width,
                    height = height,
                    points = points
                },
                energy_source =
                {
                    type = "electric",
                    usage_priority = "primary-input"
                },
                energy_consumption = "1W",
                categories = bad_apple_categories
            }
        })
    end
end

-- Filled squares: filled tiles wit hside size of 2^n (min: 4, max: 512), although 4 and 512 are unlikely to be used

local filled_square_prototypes = {}

for i = 0, 7 do
    local side_size = 4 * math.pow(2, i)
    table.insert(filled_square_prototypes,
        {
            type = proto_type,
            name = "bad-apple-tile-" .. side_size,
            localised_name = { 'item-name.bad-apple-tile', tostring(side_size) },
            localised_description = { 'item-description.bad-apple-tile' },
            hidden_in_factoriopedia = true,
            take_result = "bad-apple-starter",
            background_border_color = border_color,
            sprite = equipment_sprite_definition,
            shape =
            {
                type = "full",
                width = side_size,
                height = side_size,
            },
            energy_source =
            {
                type = "electric",
                usage_priority = "primary-input"
            },
            energy_consumption = "1W",
            categories = bad_apple_categories
        })
end

data:extend(filled_square_prototypes)

data:extend({
    -- Item used to start the animation
    {
        type = proto_type,
        name = "bad-apple-starter",
        sprite = equipment_sprite_definition,
        shape =
        {
            type = "full",
            width = chunk_size,
            height = chunk_size,
        },
        energy_source =
        {
            type = "electric",
            usage_priority = "primary-input"
        },
        energy_consumption = "1W",
        categories = bad_apple_categories
    },
    {
        type = "item",
        name = "bad-apple-starter",
        icon = "__BadAppleEquipment__/graphics/icons/part-electronic-storage.png",
        place_as_equipment_result = "bad-apple-starter",
        subgroup = "equipment",
        order = "a[energy-source]-b[fission-reactor]",
        inventory_move_sound = item_sounds.reactor_inventory_move,
        pick_sound = item_sounds.reactor_inventory_pickup,
        drop_sound = item_sounds.reactor_inventory_move,
        stack_size = 20,
        weight = 0.25 * tons
    },
    {
        type = "recipe",
        name = "bad-apple-starter",
        ingredients = { { type = 'item', name = 'iron-plate', amount = 1 } },
        results = { { type = 'item', name = 'bad-apple-starter', amount = 1 } },
        auto_recycle = false,
    },
    -- The item for viewing
    {
        type = "armor",
        name = "bad-apple-viewer",
        icon = "__base__/graphics/icons/display-panel.png",
        subgroup = "armor",
        order = "d[power-armor]",
        inventory_move_sound = item_sounds.mechanical_inventory_move,
        pick_sound = item_sounds.mechanical_inventory_pickup,
        drop_sound = item_sounds.mechanical_inventory_move,
        stack_size = 1,
        infinite = true,
        equipment_grid = "bad-apple",
        open_sound = sounds.armor_open,
        close_sound = sounds.armor_close
    },
    {
        type = "recipe",
        name = "bad-apple-viewer",
        ingredients = { { type = 'item', name = 'iron-plate', amount = 1 } },
        results = { { type = 'item', name = 'bad-apple-viewer', amount = 1 } },
        auto_recycle = false,
    },
    -- The grid prototype
    {
        type = 'equipment-grid',
        name = 'bad-apple',
        width = width,
        height = height,
        equipment_categories = bad_apple_categories
    },
    -- Equipment category
    {
        type = 'equipment-category',
        name = 'bad-apple'
    }
})
