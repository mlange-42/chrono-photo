# Preparing video material

Before this tutorial, you may want to read
[Recording material for chrono-photo](tutorial_recording.md).

Next tutorial: [Creating chrono-photos](tutorial_photos.md)

Currently, `chrono-photo` does not yet direct processing of video files.
Therefore, videos have to be converted to image sequences using a 3rd party tool.

In this tutorial, video pre-processing to images is described for
[FFmpeg](https://www.ffmpeg.org/),
[Shortcut](https://shotcut.org/),
[VLC media player](https://www.videolan.org/vlc/index.html) and
[Blender](https://www.blender.org/),
which are all open source software.

Actually, this tutorial helps to decide for a tool, and links to respective 3rd party tutorials.

**Content**

* [FFmpeg](#ffmpeg)
* [Shortcut](#shortcut)
* [VLC media player](#vlc-media-player)
* [Blender](#blender)

## FFmpeg

[FFmpeg](https://www.ffmpeg.org/) is a command line application for video processing.
As such, it is easy to use while offering no graphical user interface.

Download FFmpeg from [here](https://www.ffmpeg.org/).
Follow the installation instructions.

[A short but sufficient tutorial](https://averagelinuxuser.com/convert-video-to-images-with-ffmpeg-in-linux/)
for converting a video to an image sequence using FFmpeg.

## Shortcut

[Shortcut](https://shotcut.org/) is an open source video editing and cutting software.
Among the altenatives described here, it probably offers the best trade-off between
convenience, control, and a flat learning curve.

Download FFmpeg from [here](https://shotcut.org/).
Follow the installation instructions.

[A short video tutorial](https://www.youtube.com/watch?v=ji2-31r_C2Y)
for converting a video to an image sequence using Shortcut.

## VLC media player

[VLC media player](https://www.videolan.org/vlc/index.html) offers very basic video to image conversion via the user interface,
and some more control via the command line. 
If even more control is required, it is recommended to use Blender
or a dedicated video editing software of your choice (e.g. Shortcut).

Download VLC media player from [here](https://www.videolan.org/vlc/index.html).
Follow the installation instructions.

[A complete tutorial](https://averagelinuxuser.com/video-to-images-with-vlc-media-player/)
for converting a video to an image sequence using VLC's GUI.

Section 3 of [this post](https://www.raymond.cc/blog/extract-video-frames-to-images-using-vlc-media-player/)
explains how to do it using VLC via the command line.

## Blender

[Blender](https://www.blender.org/) is a full-featured 3D creation software, but it also has a powerful
"Video Sequencer", which we will use here.
However, Blender is so powerful and feature-rich that it may be overwhelming for people new to it.

Download Blender from [here](https://www.blender.org/).
Follow the installation instructions.

[A short video tutorial](https://www.youtube.com/watch?v=gAw6ZWO7FOY)
for converting a video to an image sequence using Blender.

#### To next tutorial: [Creating chrono-photos](tutorial_photos.md)