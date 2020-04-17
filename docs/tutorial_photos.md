# Creating chrono-photos

Previous tutorials:
[Recording material for chrono-photo](tutorial_recording.md) and [Preparing video material](tutorial_prepare.md).

Next tutorial: [Creating chrono-videos](tutorial_videos.md).

----

This tutorial explains how to use `chrono-photo` to create a chronophotography from an image sequence.
We assume that you have such an image sequence ready, e.g. obtained by following the tutorials linked above.

File extensions are for Windows. On Linux or Mac OSX, they need to be adapted accordingly.
Particularly Windows batch files (`.bat`) need to be replaces by Unix shell scripts (`.sh`). 

See the **[Command line options](options.md)** documentation file for detailed explanation of all available options.

**Content**
* [Working directory](#working-directory)
* [Printing help message](#printing-help-message)
* [Most simple command](#most-simple-command)
* [Fast algorithm for large projects](#fast-algorithm-for-large-projects)
* [Tweaking algorithm parameters](#tweaking-algorithm-parameters)
* [Camera shake reduction](#camera-shake-reduction)
* [Summary](#summary)

## Working directory

For this tutorial, we assume the following structure for your working directory:
```
root/
├── images/
│   ├── image-0000.jpg
│   ├── image-0001.jpg
│   └── ...
├── output/
└── chrono-photo.exe
``` 
> _Note:_ You can copy the `chrono-photo` executable anywhere for use with convenient paths.
No further files from the installation directory are required.

The above structure is not required, we assume it just for convenient command line usage.
E.g., input images and output folder can be in completely different locations.

## Printing help message

To print help for `chrono-photo`, run
```
chrono-photo -h
```
or, for more comprehensive help texts
```
chrono-photo --help
```

## Most simple command

We start by processing the images using standard parameters. 

In `root/`, create a file `example-01.bat` and copy the following lines there:
```
chrono-photo ^
  --pattern "images/*.jpg" ^
  --output output/out.jpg
```
Here, we only specify the search pattern for input files, as well as the output file path.

> _Note:_ The ^ at the end of each line is required for breaking commands into multiple lines (at least on Windows).

Run the file from directory `root/`:
```
C:\...\root>example-01
```

## Fast algorithm for large projects

For large projects with a lot of input images, it is recommended to start with the faster simple algorithm
by adding `--mode darker` or `--mode lighter`. 
Use the former if the moving object is darker than the background, and the latter if it is brighter.

In many cases where the background is relatively homogeneous (e.g. sky),
and the moving objects can be easily identified by brightness,
this much faster algorithm yields already sufficient results.

Additionally, in this example we enable debug output:
```
chrono-photo ^
  --pattern "images/*.jpg" ^
  --output output/out.jpg ^
  --mode darker ^
  --debug
```

This command should complete in approx. 1/10th of the time required for the first example.

## Tweaking algorithm parameters

We return to the default algorithm by using `--mode outlier`, or just omitting `--mode`.

Further, we add a few more options:
```
chrono-photo ^
  --pattern "images/*.jpg" ^
  --output output/out.jpg ^
  --output-blend output/out-blend.jpg ^
  --mode outlier ^
  --threshold abs/0.05/0.2 ^
  --background random ^
  --outlier extreme ^
  --debug
```

Here, `--output-blend` creates a debug image showing the "algorithm's view" of the images, in greyscale.

The next four option specify the algorithm and is't details:
We want to use the outlier algorithm, with an "absolute" threshold range
(actually, it is relative to total color range 0 - 255).
Selection between background pixels will be done randomly,
while the most extreme outlier is selected in case outliers are found.

## Camera shake reduction

[TODO]

## Summary

[TODO]

----

#### Next tutorial: [Creating chrono-videos](tutorial_videos.md)