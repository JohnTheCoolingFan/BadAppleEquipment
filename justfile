#!/usr/bin/env just --justfile

alias bm := buildmod
alias ba := buildall
alias g := generate
alias dv := download-video
alias pi := process-img
alias dvpi := download-and-process
alias a := all

default:
    @just --list

buildmod:
    # https://github.com/JohnTheCoolingFan/rfmp
    # Feel free to replace with your favorite method of buildign the mod files
    cd BadAppleEquipment; \
        rfmp -i "${FACTORIO_MODS_HOME:-../output/}"

generate:
    cargo run --release
    cp output/frames-tree.lua BadAppleEquipment/generated/frames-tree.lua
    cp output/more-than-two-tiles.lua BadAppleEquipment/generated/more-than-two-tiles.lua

reconstruct-frame FRAME='75':
    RECONSTRUCT_FRAME={{FRAME}} cargo run --release

buildall: generate buildmod

process-img DOWNSCALE='1':
    mkdir -p images/{{DOWNSCALE}}x
    ffmpeg -i BadApple.webm \
        -f lavfi -i color=gray:s=$((480/{{DOWNSCALE}}))x$((360/{{DOWNSCALE}})) \
        -f lavfi -i color=black:s=$((480/{{DOWNSCALE}}))x$((360/{{DOWNSCALE}})) \
        -f lavfi -i color=white:s=$((480/{{DOWNSCALE}}))x$((360/{{DOWNSCALE}})) \
        -filter_complex "[0:v]scale=$((480/{{DOWNSCALE}}))x$((360/{{DOWNSCALE}})),threshold" \
        images/{{DOWNSCALE}}x/frame%04d.png

download-video:
    yt-dlp -o BadApple.webm https://youtu.be/FtutLA63Cp8

download-and-process: download-video process-img

all: download-and-process buildall
