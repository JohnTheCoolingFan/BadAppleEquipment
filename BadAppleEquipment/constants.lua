-- Values reused in multiple stages

local C = {
    width = 480,
    height = 360,

    chunk_size = 8,

    frame_count = 6575,

    equipment_prototype_type = 'belt-immunity-equipment',

    grid_cell_scale = settings.startup['bad-apple-grid-scale'].value,
}

if settings.startup['bad-apple-show-borders'] then
    C.border_color = { 89 / 255, 89 / 255, 89 / 255 }
end

return C
