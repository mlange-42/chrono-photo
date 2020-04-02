# chrono-photo

Chronophotography command line tool and library in [Rust](https://www.rust-lang.org/).

![A simple Chronophotography example](https://user-images.githubusercontent.com/44003176/77975353-236da480-72fa-11ea-9ff9-5c110895fe5d.jpg)
<sup>_A simple Chronophotography example_</sup>

This tool creates chrono-photos like 
[Xavi Bou's "Ornithographies"](http://www.xavibou.com/) 
from video footage or photo series.

_Warning:_ This project is in a very experimental state.
So far, only selection by lightest or darkest pixel are supported. 
However, the image above shows a proof of concept for the intended default algorithm, 
which is based on outlier detection (see section [How it works](#how-it-works) for details). 

## Command line tool

### Installation

* Download the [latest binaries](https://github.com/mlange-42/chrono-photo/releases).
* Unzip somewhere with write privileges (only required for running examples in place).

### Usage

* Try the example batch files in sub-directory [`/cmd_examples`](/cmd_examples).
* To view the full list of options, run `chrono-photo --help`

## Library / crate

To use this crate as a library, add the following to your `Cargo.toml` dependencies section:
```
chrono-photo = { git = "https://github.com/mlange-42/chrono-photo.git" }
```

## How it works

The principle idea is to stack all images to be processes, and analyze them pixel by pixel.

Given a typical use case like a moving object in front of a static background, 
a certain pixel will have very similar colors in most images. 
In one or a few images, the pixel's value may be different, as it shows the object rather than the background.
I.e. among the pixel's color from all images, these images would be outliers.

The idea now is to use these outliers for the output image, if they exist for a certain pixel, 
or a non-outlier if they don't.

### Technical realization

Loading a large number of high resolution images into memory at once is not feasible. 

Therefore, before actual processing, the time-stack of images with (x, y) coordinates
is converted into a number of temporary files, each containing data in (x, t) coordinates.

For example, the first temporary file contains the first row of pixels from each image.

Using these temporary files, all images can be processes row by row, without overloading memory.