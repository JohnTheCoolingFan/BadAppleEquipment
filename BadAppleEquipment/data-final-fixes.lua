local constants = require("constants")
local width = constants.width
local height = constants.height

local chunk_size = constants.chunk_size

local item_sounds = require("__base__.prototypes.item_sounds")

data.raw['equipment-grid']['medium-equipment-grid'].width = width
data.raw['equipment-grid']['medium-equipment-grid'].height = height

data.raw['utility-sprites']['default'].equipment_slot.scale =
    data.raw['utility-sprites']['default'].equipment_slot.scale / 20
--data.raw['utility-constants']['default'].equipment_default_background_border_color = { 89 / 255, 89 / 255, 89 / 255 }

local border_color = nil

if settings.startup['bad-apple-show-borders'] then
    border_color = { 89 / 255, 89 / 255, 89 / 255 }
end

--local used_tiles = require("used-tiles")
local frames_tree = require("generated.frames-tree")

local max_width = constants.width
local max_height = constants.height

local function descend_tree(node, points, anchor_x, anchor_y, side_size)
    if anchor_x > max_width or anchor_y > max_height then
        return
    end
    if type(node) == "number" then
        return
    end
    if type(node) == "string" then
        local part1 = string.sub(node, 1, 8)
        local part0 = string.sub(node, 9, 16)
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
        descend_tree(child, points, new_anchor_x, new_anchor_y, new_side_size)
    end
end

for framenum, node in pairs(frames_tree) do
    local points = {}

    descend_tree(node, points, 0, 0, 512)

    if #points > 0 then
        data:extend({
            {
                type = "generator-equipment",
                name = "bad-apple-tile-" .. framenum,
                take_result = "bad-apple-starter",
                background_border_color = border_color,
                sprite =
                {
                    filename = "__base__/graphics/equipment/fission-reactor-equipment.png",
                    width = 1,
                    height = 1,
                    priority = "medium",
                    scale = 0.005
                },
                shape =
                {
                    type = "manual",
                    width = max_width,
                    height = max_height,
                    points = points
                },
                energy_source =
                {
                    type = "electric",
                    usage_priority = "primary-output"
                },
                power = "1W",
                categories = { "armor" }
            }
        })
    end
end
data:extend({
    {
        type = "generator-equipment",
        name = "bad-apple-starter",
        sprite =
        {
            filename = "__base__/graphics/equipment/fission-reactor-equipment.png",
            width = 1,
            height = 1,
            priority = "medium",
            scale = 0.005
        },
        shape =
        {
            type = "full",
            width = chunk_size,
            height = chunk_size,
        },
        energy_source =
        {
            type = "electric",
            usage_priority = "primary-output"
        },
        power = "1W",
        categories = { "armor" }
    },
    {
        type = "item",
        name = "bad-apple-starter",
        icon = "__base__/graphics/icons/fission-reactor-equipment.png",
        place_as_equipment_result = "bad-apple-starter",
        subgroup = "equipment",
        order = "a[energy-source]-b[fission-reactor]",
        inventory_move_sound = item_sounds.reactor_inventory_move,
        pick_sound = item_sounds.reactor_inventory_pickup,
        drop_sound = item_sounds.reactor_inventory_move,
        stack_size = 20,
        weight = 0.25 * tons
    }
})

for i = 0, 7 do
    local side_size = 4 * math.pow(2, i)
    data:extend({
        {
            type = "generator-equipment",
            name = "bad-apple-tile-full-" .. side_size,
            take_result = "bad-apple-starter",
            background_border_color = border_color,
            sprite =
            {
                filename = "__base__/graphics/equipment/fission-reactor-equipment.png",
                width = 1,
                height = 1,
                priority = "medium",
                scale = 0.005
            },
            shape =
            {
                type = "full",
                width = side_size,
                height = side_size,
            },
            energy_source =
            {
                type = "electric",
                usage_priority = "primary-output"
            },
            power = "1W",
            categories = { "armor" }
        },
        --[[
        {
            type = "item",
            name = "bad-apple-tile-full-" .. side_size,
            icon = "__base__/graphics/icons/fission-reactor-equipment.png",
            place_as_equipment_result = "bad-apple-tile-full-" .. side_size,
            subgroup = "equipment",
            order = "a[energy-source]-b[fission-reactor]",
            inventory_move_sound = item_sounds.reactor_inventory_move,
            pick_sound = item_sounds.reactor_inventory_pickup,
            drop_sound = item_sounds.reactor_inventory_move,
            stack_size = 20,
            weight = 0.25 * tons
        },
        ]]
    })
end
