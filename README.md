# chrono-photo

Chronophotography command line tool and library in [Rust](https://www.rust-lang.org/).

This tool helps to create chrono-photos like 
[Xavi Bou's "Ornithographies"](http://www.xavibou.com/index.php/project/ornitographies/) 
from photo series or ([later](#how-to-prepare-videos)) video footage.

* **[Download binaries](https://github.com/mlange-42/chrono-photo/releases/)**
* [Sources on GitHub](https://github.com/mlange-42/chrono-photo)

<p align="center">
<img src="https://user-images.githubusercontent.com/44003176/77975353-236da480-72fa-11ea-9ff9-5c110895fe5d.jpg" alt="A simple Chronophotography example" width="750" /><br/>
<sup><i>A simple Chronophotography example</i></sup>
</p>

**Content**
* [Installation](#installation)
* [Getting started](#getting-started)
* [Tutorials](#tutorials)
* [Examples](#examples)
* [How it works](#how-it-works)
* [Command line options](#command-line-options)
* [How to prepare videos](#how-to-prepare-videos)
* [Library / crate](#library--crate)

## Installation

* Download the [latest binaries](https://github.com/mlange-42/chrono-photo/releases).
* Unzip somewhere with write privileges (only required for running examples in place).

## Getting started

* Try the example batch files in sub-directory [/cmd_examples](https://github.com/mlange-42/chrono-photo/tree/master/cmd_examples)
(run `create-test-data` before to generate the required example data). 
* See section [Command line options](#command-line-options) or run `chrono-photo --help` to view the full list of options.
* See [docs/options.md](docs/options.md) for a detailed explanation of all options.
* See section [How it works](#how-it-works) for an explanation of the algorithm .

## Tutorials

* [Recording material for chrono-photo](docs/tutorial_recording.md)
* [Preparing video material](docs/tutorial_prepare.md)
* [Creating chrono-photos](docs/tutorial_photos.md)
* [Creating chrono-videos](docs/tutorial_videos.md)

## Examples

Minimal example:
```
chrono-photo ^
  --pattern "input/image-*.jpg" ^
  --output output/out.jpg
```

> _Note 1:_ The ^ at the end of each line is required for breaking commands into multiple lines (at least on Windows).

> _Note 2:_ You can copy the chrono-photo executable anywhere to use it with convenient paths,
in case adding it to the PATH environmental variable is not an option. 
No further files from the installation directory are required to run chrono-photo.

For more options, see [Command line options](#command-line-options).

## How it works

The basic algorithm to use is selected by option `--mode`.

Three different modes are available: `darker`, `lighter` and `outlier`. 
The first two modes simply select the darkest or brightest value found for a certain pixel among all images.
Calculations are very fast, but control over the result is limited.

The core algorithm of this tool, which is explained in detail below,
is based on outlier detection (`--mode outlier`).
This method offers great control over the result and can handle a far wider range of scenarios.
However, it is also computationally more expensive.

The principle idea is to stack all images to be processes, and analyze the entire stack pixel by pixel
(but see also section [Technical realization](#technical-realization)).

Given a typical use case like a moving object in front of a static background, 
a certain pixel will have very similar colors in most images. 
In one or a few images, the pixel's value may be different, as it shows the object rather than the background.
I.e. among the pixel's color from all images, these images would be outliers.

The idea now is to use these outliers for the output image, if they exist for a certain pixel,
or a non-outlier if they don't. Actually, the algorithm blends the outlier into the background depending on "how much of an outlier" it is.

### Outlier detection

Outlier detection in the current version uses multi-dimensional distance to the median,
and an absolute or relative (lower) threshold provided via option `--threshold` (default: abs. 0.05; `--threshold abs/0.05/0.2`). 
The _absolute_ threshold (recommended, typically < 1) is relative to the per-band color range (i.e. fraction of range [0, 255] for 8 bits per color band),
while the _relative_ threshold (typically > 1) is relative to the inter-quartile range in each band/dimension.

A pixel value is categorized as an outlier if it's distance from the median is at least the threshold.
If multiple outliers are found, one is selected according the description in 
[Pixel selection among outliers](#pixel-selection-among-outliers).

If the distance of the outlier to the median is between lower and upper threshold (the two numbers in `--threshold abs/0.05/0.2`),
the pixel color is blended between background and outlier (linear). 
If the distance is above the upper threshold, the outlier's color is used without blending.

#### Pixel selection among outliers

If only one outlier is found for a pixel, it is used as the pixel's value.

If more than one outlier is found for a pixel, different methods can be used to select among them 
via **option `--outlier`**:
* `extreme`: use the most extreme outlier (the default value).
* `average`: use the average of all outliers.
* `forward`, `backward`: progressively blends all outliers over the background, starting with the first or last, respectively.
* `first`: use the first outlier found.
* `last`: use the last outlier found.

#### Background pixel selection

If no outliers are found for a pixel (or for blending), different methods can be used to select the 
pixel's value via **option `--background`**:
* `random`: Use a randomly selected pixel value, selected among all non-outlier images. The default value, but may result in a noisy image.
* `first`: Use the pixel value from the first non-outlier image.
* `average`: Use the average pixel value of all non-outlier images. Can be used for blurring, but may result in banding for low contrast backgrounds.
* `median`: Use the median pixel value of all images (including outliers!). May result in banding for low contrast backgrounds.

#### Parameter selection

Finding the best options for pixel selection, as well as an outlier threshold that fits the noise in the input images,
may require some trial and error.

In addition to inspection of the produced image, use option `--output-blend <path>` to write a greyscale
image showing which pixels were filled based on outliers (greyscale blend value), and which were not (black).

If there are black pixels inside the moving object(s), the outlier threshold(s) should be decreased. 
On the other hand, if there are white or grey pixels outside the moving object(s), the threshold(s) should be increased
(may happen due to too much image noise, an insufficiently steady camera, or due to motion in the background).

### Technical realization

Holding a large number of high resolution images in memory at the same time is not feasible. 

Therefore, before actual processing, the time-stack of images with (x, y) coordinates
is converted into a number of temporary files, each containing data in (x, t) coordinates.
For example, the first temporary file contains the first row of pixels from each image.

Using these temporary files, all images can be processes row by row, without overloading memory, as explained above.

Actually, the above description is a simplification. Option `--slice` provides control
over how much data from each image goes into each temporary file. The option accepts different forms.
Examples:
* `--slice rows/4`: Writes 4 rows of each image into each time slice.
* `--slice pixels/1000`: Writes 1000 pixels of each image into each time slice.
* `--slice count/100`: Internally determines the amount of data written, in order to create a total of 100 time slices.

The default (`rows/4`) should be sufficient for most scenarios. 

**Higher values** can however be used to reduce the number of temporary files created, 
and to slightly increase the efficiently of compression of these files.

**Lower values** may be necessary when processing really huge numbers of images.
During the actual processing, one entire time slice file is loaded into memory at a time.
As an example, processing 100'000 frames in Full HD resolution with `--slice rows/1` requires loading
`frames * width` pixels (200 megapixels) into memory, which are approximately 600 MB. 
By writing, e.g., only half a row per file (`--slice pixels/960` for Full HD),
memory usage can also be reduces to the half, while producing twice as many temporary files.

## Command line options

**For detailed explanation, see [docs/options.md](docs/options.md)**.

```
Command-line tool for combining images into a single chrono-photograph or chrono-video.

Use `chrono-photo -h`     for help, or
    `chrono-photo --help` for more detailed help.

For more documentation and explanation of the algorithm, see the GitHub repository:
     https://github.com/mlange-42/chrono-photo

USAGE:
    chrono-photo [FLAGS] [OPTIONS] --output <path> --pattern <pattern>

FLAGS:
        --debug      Print debug information (i.e. parsed cmd parameters)
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --background <bg>           Background pixel selection mode (first|random|average|median). Optional, default
                                    'random'. Used with `--mode outlier` only
    -c, --compression <comp/lev>    Compression mode and level (0 to 9) for time slices (gzip|zlib|deflate)[/<level>].
                                    Used with `--mode outlier` only. Optional, default 'gzip/6'
        --fade <fade>               Frame fading. Optional, default None. Format:
                                    (clamp|repeat)/(abs|rel)/(f1,v1)/(f2,v2)[/(f,v)...]
    -f, --frames <frames>           Frames to be used from those matching pattern: `start/end/step`. Optional. For
                                    default values, use `.`, e.g. `././2`
    -m, --mode <mode>               Pixel selection mode (lighter|darker|outlier). Optional, default 'outlier'
    -l, --outlier <mode>            Outlier selection mode in case more than one outlier is found
                                    (first|last|extreme|average|forward|backward). Optional, default 'extreme'. Used
                                    with `--mode outlier` only
    -o, --output <path>             Path to output file
        --output-blend <path>       Path of output image showing which pixels are outliers (blend value). Used with
                                    `--mode outlier` only
    -p, --pattern <pattern>         File search pattern. ** MUST be quoted on Unix systems! **
    -q, --quality <quality>         Output image quality for JPG files, in percent. Optional, default '95'
        --sample <sample>           Restricts calculation of median and inter-quartile range to a sub-sample of input
                                    images. Use for large amounts of images to speed up calculations. Optional. Used
                                    with `--mode outlier` only
        --shake <r1/r2>             Camera shake reduction parameters. Optional, default none. Format: `anchor-
                                    radius/shake-radius`
        --shake-anchors <x/y>...    Camera shake reduction anchors. Optional, default none. Format: `x1/y1 [x2/y2
                                    [...]]`
    -s, --slice <slice>             Controls slicing to temp files (rows|pixels|count)/<number>. Used with `--mode
                                    outlier` only. Optional, default 'rows/4'
    -d, --temp-dir <path>           Temp directory. Used with `--mode outlier` only. Optional, default system temp
                                    directory
        --threads <num>             Number of threads. Optional, default equal to number of processors
    -t, --threshold <thresh>        Outlier threshold mode (abs|rel)/<lower>[/<upper>]. Optional, default
                                    'abs/0.05/0.2'. Used with `--mode outlier` only
        --video-in <frames>         Video input frames. Frames to be used per video frame: `start/end/step`. Optional
        --video-out <frames>        Video output frames. Range and step width of video output frames: `start/end/step`.
                                    Optional
        --video-threads <num>       Number of threads for parallel video frame output. Optional, default equal to
                                    number of processors. Limiting this may be required if memory usage is too high
    -w, --weights <w>...            Color channel weights (4 values: RGBA) for distance calculation. Optional, default
                                    '1 1 1 1'
```

## How to prepare videos

See also the tutorial on [Preparing video material](docs/tutorial_prepare.md).

There is no support for direct video file processing _yet_.

To process videos, they have to be converted into a sequence of images by a third party tool.
E.g. with the open source software [Blender](https://www.blender.org/),
using it's _Video Sequencer_. The required settings are shown in the image below
(particularly, see 'Output' in the bottom-left corner).

![Blender-VideoSequencer](https://user-images.githubusercontent.com/44003176/78508454-58a94500-7787-11ea-9e55-675e88cf14d7.PNG)
_Blender with_ Video Sequencer _view (right part) and required output settings (bottom left).
To start rendering, click 'Render Animation' in menu 'Render' (top-most menu bar) or press Ctrl+F12._

## Library / crate

To use this crate as a library, add the following to your `Cargo.toml` dependencies section:
```
chrono-photo = { git = "https://github.com/mlange-42/chrono-photo.git" }
```

_Warning:_ The API is still incomplete and highly unstable, so be prepared for frequent changes. 
Any help to stabilize the API is highly appreciated.

For the latest development version, see branch [`dev`](https://github.com/mlange-42/chrono-photo/tree/dev).
