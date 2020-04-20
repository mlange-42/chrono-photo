# Command line options

**Content**
* [Input and output](#input-and-output)
  * [--pattern](#--pattern) &nbsp; [--output](#--output) &nbsp; [--output-blend](#--output-blend) &nbsp;
[--temp-dir](#--temp-dir) &nbsp;  [--frames](#--frames) &nbsp;  [--quality](#--quality)
* [Algorithm](#algorithm)
  * [--mode](#--mode) &nbsp; [--threshold](#--threshold) &nbsp; [--outlier](#--outlier) &nbsp;
[--background](#--background) &nbsp; [--weights](#--weights) &nbsp; [--fade](#--fade)
* [Video creation](#video-creation)
  * [--video-in](#--video-in) &nbsp; [--video-out](#--video-out)
* [Camera shake reduction](#camera-shake-reduction)
  * [--shake](#--shake) &nbsp; [--shake-anchors](#--shake-anchors)
* [Performance](#performance)
  * [--threads](#--threads) &nbsp; [--video-threads](#--video-threads) &nbsp; [--shake-threads](#--shake-threads) &nbsp; [--sample](#--sample) &nbsp; [--compression](#--compression) &nbsp; [--slice](#--slice)
* [Misc](#misc)
  * [--debug](#--debug) &nbsp; [--wait](#--wait)

## Input and output

#### `--pattern`

_Required._
Search pattern for input files (glob-style).

_**Note:**_ The pattern **MUST be quoted on Unix** systems.

Examples:
```
--pattern "path/to/*.jpg"
--pattern "image-*.jpg"
```
The files found are used in lexical order.

#### `--output`

_Required._ Output file path. File format is determined from the extension. Supported formats are JPG, PNG and TIF. 
For a list of formats potentially supported, but not tested, see crate [`image`](https://docs.rs/image/).

Examples:
```
--output path/to/out.png
```

#### `--output-blend`

_Optional, used with `--mode outlier` only._

Output path for the greyscale image showing the algorithm's outlier detections. See [`--output`](#--output) for details.

_Default:_ No output of outlier image.

#### `--temp-dir`

_Optional, used with `--mode outlier` only._

Temporary directory for storing time slice files. 
Files are delected after processing, while the directory is not. 
If the directory does not exist, but the parent directory exists, it is created.
If the parent directory does not exist, the program exits with an error.

_Default:_ `<system-temp>/chrono-photo/`

#### `--frames`

_Optional._ A range of frames in the format `start/end/step`, where `end` is exclusive.

The `.` character can be used as placeholder.

Examples:
```
--frames ././2     -> Use every second frame (of all frames; no start or end given)
--frames ./20/.    -> Use the first 20 frames (zero-based indexing, end exclusive)
```

_Default:_ Use all images found for [`--pattern`](#--pattern).

#### `--quality`

_Optional._ Output quality for JPEG images, in percent (1 - 100).

Examples:
```
--quality 100
```

_Default:_ `95`

## Algorithm

#### `--mode`

_Optional._ Foreground pixel detection mode. One of `outlier|darker|lighter`.

_Default:_ `outlier`

#### `--threshold`

_Optional._ Outlier distance to median threshold and blend distance in format `(abs|rel)/min[/max]`.
Colors closer to the median than `min` are considered background.
Colors between `min` and `max` are blended over the background linearly.
Colors more distant to the median then `max` are blended over the background with 100% (replace, but see [--fade](#--fade)).

An **absolute** threshold relates to the total color range, e.g. fractions of 255 for 8 bit per channel images.
E.g. `abs/0.1/0.2` for 8 bits images blends for distances between 25.5 and 51 (0.1 * 255, 0.2 * 255).

A **relative** threshold relates to the inter-quartile range (IQR, difference between upper and lower quartile)
of the pixel's colors (separately for each channel / dimension).
E.g. `rel/2/4` blends for distances between twice the IQR to four times the IQR.

Examples:
```
--threshold abs/0.05/0.2
--threshold rel/3.0/5.0
```

_Default:_ `abs/0.05/0.2`

#### `--outlier`

_Optional, used with `--mode outlier` only._

Outlier selection mode. Determines how selection between multiple outliers is performed.

This parameter is particularly useful when several movement trails overlap,
or when a moving object is relatively slow compared to the recording frame rate, and thus overlaps itself. 
For non-overlapping trails, the default `extreme` should be the best.

* `extreme`: use the most extreme outlier (the default value).
* `average`: use the average of all outliers.
* `forward`, `backward`: progressively blends all outliers over the background, starting with the first or last, respectively.
* `first`: use the first outlier found.
* `last`: use the last outlier found.

_Default:_ `extreme`

#### `--background`

_Optional, used with `--mode outlier` only._

Background color selection mode. Determines how selection between non-outlier colors is performed.

* `random`: Use a randomly selected pixel value, selected among all non-outlier images. The default value, but may result in a noisy image.
* `first`: Use the pixel value from the first non-outlier image.
* `average`: Use the average pixel value of all non-outlier images. Can be used for motion blur, but may result in banding for low contrast backgrounds.
* `median`: Use the median pixel value of all images (including outliers!). May result in banding for low contrast backgrounds.

_Default:_ `random`

#### `--weights`

_Optional._ Color channel weights for outlier detection in format `r g b a`.
Must be four numeric values (typically 0 - 1), irrespective of the actual number of color channels in the input images.

Examples:
```
--weights 0 0 1 0              -> Outlier analysis is based only on the blue channel
--weights 1.0 0.5 0.5 0.0      -> Outlier analysis is based on more weight on red than on blue and green
```

_Default:_ `1 1 1 1`

#### `--fade`

_Optional._ Allows for fading outlier blending over frames. Format `(clamp|repeat)/(abs/rel)/f,v/f,v[/f,v...]`

Parts between `/` are:
1. Fading mode: `clamp` or `repeat`. Specifies how frames outside the given fade transition are treated.
1. `abs` or `rel`: specifies where the frames given in the `frame,value` parts relate to. 
`abs` is relative to the first frame in the entire sequence (forward). 
`rel` is relative to the last actually processed frame (backward).
See the examples.
1. At least two blend value pairs in the format `frame,value` (without additional spaces!).

**Must be in strictly increasing frame order!**

Between the specified frames, values are interpolated linearly. Values should be between 0 and 1.

Examples:
```
--fade repeat/rel/0,1/9,0   -> Repeated, fading-out movement trails over 10 'instances' of a moving object
                               Would move when use for video output
```

_Default:_ No fading.

## Video creation

For video creation, at least one of the two options `--video-in` and `--video-out` must be provided.

For video output, the frame counter is appended to the file name of the path provided by option `--output`.
E.g. `out.jpg` becomes `out-00000.jpg`, `out-00001.jpg`, etc.

#### `--video-in`

_Optional._ Frame range of input images per video frame, relative to the current video frame. 
Format `start/end/step`, where `end` is exclusive. Use `.` as placeholder.

If `start` and `end` are given, it results in a 'moving window' over all images. 
If `start` or `end` are placeholders, the frames range from the very start, 
or to the very end (of the entire image sequence), respectively.

Examples:
```
--video-in 0/25/.     -> Each video frame will contain 25 frames. 
                         Results in a 'moving trails'.
--video-in ./1/.      -> Each video frame will contain all images up to the current frame.
                         Results in a 'growing trail'.
--video-in 0/50/5     -> Each video frame will contain every 5th image of the given range.
                         Results in the impression of the moving object 'following itself' multiple times 
                         in a certain distance. 
                         Requires start and end to be specified in order to work as intended!
```

Everything here refer to the frames left _after_ selection through option [`--frames`](#--frames)!

_Default:_ No video output, or `././.` if `--video-out` is specified.

#### `--video-out`

_Optional._ Frame range of the video. in format `start/end/step`, where `end` is exclusive. Use `.` as placeholder.

The actual frame range will most likely be larger, as it is automatically extended
to cover the per-frame offsets given by option `--video-in`.

In most cases, it is not necessary to set this option and just use `--video-in`.

Everything here refer to the frames left _after_ selection through option [`--frames`](#--frames)!

_Default:_ No video output, or `././.` if `--video-in` is specified.

## Camera shake reduction

To enable camera shake reduction, both of the following options must be supplied.
By default, no camera shake reduction is applied.

If camera shake is detected, images are cropped by the amount of shake for correction.
Thus, the output image will be slightly smaller than the input images.

#### `--shake`

_Optional._ Shake anchor radius and search radius in format `anchor-radius/search-radius`.

Example:
```
--shake 10/5
```
_Default:_ No camera shake reduction.

#### `--shake-anchors`

_Optional._ Pixel coordinates of shake detection anchors in the _first_ image. 
Format `x1/y1 [x2/y2 ...]`.

Anchors are optimally placed at positions with high contrast in both directions (x, y),
like a dark corner on light background.
Also, anchors should not be occluded by a moving object in any image of the sequence.

Use an image editing software to get the exact pixel coordinates.
Origin is the top-left corner of the image.

Example:
```
--shake-anchors 1234/789 2345/890
```

_Default:_ No camera shake reduction.

## Performance

#### `--threads`

_Optional._

Number of threads to use for parallel processing.

_Default:_ Number of processors.

#### `--video-threads`

_Optional._

Number of threads to use for parallel processing for video / image sequence creation. 
Entire frames are processed in parallel. 
It may be necessary to limit the number of video threads if memory usage is too high.

_Default:_ Number of processors.

#### `--shake-threads`

_Optional._

Number of threads to use for parallel camera shake analysis. 
Entire frames are processed in parallel. 
It may be necessary to limit the number of shake threads if memory usage is too high.

_Default:_ Number of processors.

#### `--sample`

_Optional, used with `--mode outlier` only._

Specifies a sample count to reduce the number of images used for calculation of per-pixel median
and (if required) quartiles.

This option is particularly useful to speed up calculations when processing large numbers of images (thousands).

_Default:_ No sampling, use all images.

#### `--compression`

_Optional, used with `--mode outlier` only._

Compression method and level for temporary time slice files. 
Format `(gzip|zlib|deflate)[/<level>]`. 
Levels range from 0 (no compression) to 9 (slowest).

_Default:_ `gzip/6`

#### `--slice`

_Optional, used with `--mode outlier` only._

Time-slicing in the format `(rows|pixels|count)/<number>`.

_Default:_ `rows/4` (should be sufficient for most scenarios)

Holding a large number of high resolution images in memory at the same time is not feasible. 

Therefore, before actual processing, the time-stack of images with (x, y) coordinates
is converted into a number of temporary files, each containing data in (x, t) coordinates.
For example, the first temporary file contains the first row of pixels from each image.

Using these temporary files, all images can be processes row by row, without overloading memory, as explained above.

Actually, the above description is a simplification. Option `--slice` provides control
over how much data from each image goes into each temporary file. The option accepts different forms.

Examples:
```
--slice rows/4         -> Writes 4 rows of each image into each time slice.
--slice pixels/1000    -> Writes 1000 pixels of each image into each time slice.
--slice count/100      -> Internally determines the amount of data written, in order to create 
                          a total of 100 time slices.
```

## Misc

#### `--debug`

_Optional._ Switch to print the parsed command line arguments for debugging.

#### `--wait`

_Optional._ Switch to keep the terminal open and wait for user key press.
Useful when running a `.bat`, `.sh` or `.chrono` file by double click,
to let the user check for errors.
