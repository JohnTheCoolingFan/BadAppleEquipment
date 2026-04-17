# Bad Apple in Factorio equipment grid

This repository includes the rust project for preprocessing initial data, as well as the source code for the Factorio mod, except for the generated lua files.

Showcase: https://youtu.be/UgXh1dESQqw

## Licensing

The code in this repository is licensed under MIT license, except for the image in `BadAppleEquipment/graphics/icons/part-electronic-storage.png`, which comes from [malcolmriley's unused renders](https://github.com/malcolmriley/unused-renders) and is licensed under [CC Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)

## How this works

The Factorio modding API allows specifying custom shapes for the equipment, which is what is used here. Unfortunately, you specify the shapes by providing a list of x-y coordinate pairs for every filled square. Good enough when the grid's dimensions are not big and the amount of equipment is low. But this is the opposite case. Just having all frames as equipment in full resolution would require an insane amount of RAM. For my first attempt, I had to scale down the size of the frames by 3, which is shown in this video: https://youtu.be/qCHp5uNLUvY

There were also issues with transferring the processed data. The raw sequence of points resulted in close to 5 gigabytes of lua code, which is more than what fits in a single file in a regular zip archive! The first workaround was to split the data into multiple files, which worked well. Second was to use run-length encoding, which reduced the file size SIGNIFICANTLY. But none of it addressed the RAM usage.

The solution to the RAM usage was reusing similar segments of the frames. At first, the image was simply divided into 4x4 tiles, which already significantly improved the RAM situation since a lot of the tiles are either empty or full tiles. Some 4x4 tiles appeared in multiple frames, so they didn't have to be repeated for each frame. This was a good step forward, but now the game has issues rendering all of the equipment. This was solved in multiple steps:

### 1. Combine empty or filled squares

There is A LOT of either black or white square space in Bad Apple. you can reduce the number of tiles in one frame by combining empty/full tiles into larger ones. This was done by using a [quadtree](https://en.wikipedia.org/wiki/Quadtree) algorithm (custom implementation) which recursively divides the image into squares.

### 2. Ignore empty tiles

At first, the empty tiles were put into the grid as well, but since Factorio doesn't allow equipment without any points defined (try to guess why!), it had a pixel added in the top-left corner. This resulted in visual noise and unnecessary equipment bring drawn and processed by the game, so simply ignoring the empty areas was an obvious improvement.

### 3. Increasing the tile size

The initial tile size was 4x4. Why? Well, 4x4 tiles results in 16 pixels per tile, and since every pixel can be either black or white, it can be represented as a bit. And the total number of tile variations is 2^16 = 65536. This is the exact limit of prototypes Factorio can have per category. Thankfully, the actual amount of tiles at full resolution was much lower. But even with previous optimizations, the game had issues rendering the game at 30 fps or more (the framerate of the source animation). So, the tile size was increased to 8x8. One small issue:

```
Number of unique chunks: 138392
```

And that is clearly more than 65536. This would not load in game.

### 4. Combining tiles

To work around the prototype limit, at first all shaped tiles (not filled, not empty) were combined for each frame. this results in no more than 6575 prototypes, but now the RAM issue is back. The tiles were initially used to deduplicate identical image segments, but now that data is repeated in a lot of frames. So, the next step was to bring back the deduplication, while combining the unique tiles into frame-wide tiles. And that actually worked!

```
Number of unique chunks: 138392
There are 32837 tiles that are used more than once
There are 37230661 total pixels in tiles in all frames
The tiles that are shared between frames amount to 33954260 pixels
The number of pixels in unique tiles across all frames is 3276401
```

With this approach, there are 33954260 less x-y coordinate pair tables while loading the tile data. Don't be confused by the first line of the output though, it only means how many tile variants there are in all of the frames in the animation.

The final solution is to dump a list of all the tiles that are used twice or more into a lua file and have that be decoded into tile prototypes during the data stage. Then the quadtree graph is walked, and the tiles that are not in the list are "drawn" into the whole-frame tile.

In the end, the game used 5.5GB peak amount of RAM during loading, and around 4.5GB while playing the animation.

### Possible further optimizations

1. I have been told about `string.pack` and `string.unpack` which could probably be used to reduce the file size of the generated lua files, but I find that prospect a bit too confusing for me and don't know how to deal with it when you have a dynamically-sized format, like the quadtree.
2. Determining whether to put a tile into a combined frame based on a metric: would help keep the balance between execution speed and RAM usage.
3. Not erasing the frame completely. Only having to replace the tiles that are changed in the next frame could reduce the amount of equipment grid operations that are needed and improve the playback speed.

## Building

The repository contains a `justfile` which you can use with https://github.com/casey/just to automate some steps.

The full process is the following:
- `just download-video` (alias `dv`): Get the source video, 480x360 resolution, BadApple.webm. The justfile already has a yt-dlp with a youtube link to a video. If you do not want to use yt-dlp, find an alternative way to get a source video and adjust the constants in `src/main.rs` and `BadAppleEquipment/constants.lua` accordingly.
- `just process-img` (alias `pi`): Uses ffmpeg to separate video into images into the `images/1x/` folder and applies threshold to every one.
- `just generate` (alias `g`): Build and run the rust project that processed the images into lua files and also generates statistics. Look in `output/` folder if you're interested. Also copies the files into the Factorio mod files.
- `just buildmod` (alias `bm`): Uses [`rfmp`](https://github.com/JohnTheCoolingFan/rfmp) to build the Factorio mod as a zip file. Feel free to skip this if you want to use a different tool or process. By default it puts the mod file into directory `output/` or the value of `FACTORIO_MODS_HOME` environment variable.

There's a few extras:
- `just reconstruct-frame [FRAME]`: reconstructs a specific frame into `output/reconstructed_frame_FRAME.png`, defautls to 75
- `just buildall` (alias `ba`): `buildmod` and `generate` steps
- `just download-and-process` (alias `dvpi`): `download-video` and `process-img` steps
- `just all` (alias `a`): do all 4 steps of the build process
