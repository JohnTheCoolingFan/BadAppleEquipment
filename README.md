# Bad Apple in Factorio equipment grid

This repository includes the rust project for preprocessing initial data, as well as the source code for the Factorio mod, except for the generated lua files.

The repository contains a `justfile` which you can use with https://github.com/casey/just to automate some steps.

The full process is the following:
- `just download-video` (alias `dv`): Get the source video, 480x360 resolution, BadApple.webm. The justfile already has a link you can use with yt-dlp. If you do not want to use yt-dlp, find an alternative way to get a source video and adjust the constants in `src/main.rs` and `BadAppleEquipment/constants.lua` accordingly
- `just process-img` (alias `pi`): Uses ffmpeg to separate video into images into the `images/1x/` folder and applies threshold to every one.
- `just generate` (alias `g`): Build and run the rust project that processed the images into lua files and also generates statistics. Look in `output/` folder if you're interested. Also copies the files into the Factorio mod files
- `just buildmod` (alias `bm`): Uses [`rfmp`](https://github.com/JohnTheCoolingFan/rfmp) to build the Factorio mod as a zip file. Feel free to skip this if you want to use a different tool or process. By default it puts the mod file into directory `output/` or the value of `FACTORIO_MODS_HOME` environment variable

There's a few extras:
- `just reconstruct-frame [FRAME]`: reconstructs a specific frame into `output/reconstructed_frame_FRAME.png`, defautls to 75
- `just buildall` (alias `ba`): `buildmod` and `generate` steps
- `just download-and-process` (alias `dvpi`): `download-video` and `process-img` steps
- `just all` (alias `a`): do all 4 steps of the build process
