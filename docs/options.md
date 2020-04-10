# Command line options

**Content**
* [Input and output](#input-and-output)
* [Algorithm](#algorithm)
* [Video creation](#video-creation)
* [Performance](#performance)

## Input and output

#### `--pattern`

_Required._ 
Search pattern for input files (glob-style). 

Examples:
```
--pattern path/to/*.jpg
--pattern image-*.jpg
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

Output path for the mask image showing the algorithm's outlier detections. See [`--output`](#----output) for details.

#### `--temp-dir`

#### `--frames`

#### `--quality`

## Algorithm

#### `--mode`

#### `--threshold`

#### `--outlier`

#### `--background`

#### `--weights`

## Video creation

#### `--video-in`

#### `--video-out`

## Performance

#### `--sample`

#### `--slice`

#### `--compression`
