# Recording material for chrono-photo

Next tutorial: [Preparing video material](tutorial_prepare.md).

----

This tutorial covers how to record material for use with `chrono-photo`,
particularly for fast-moving subjects like birds.

**Content**
* [Photo or video?](#photo-or-video)
* [General considerations](#general-considerations)
* [What you need](#what-you-need)
* [Camera settings](#camera-settings)
* [Summary](#summary)

## Photo or video?

### Mostly videos

Fast-moving subjects require recording videos rather than photos due to the frame rate limitations
when using a camera in still photography mode. 
So you will unfortunately not be able to take advantage of your DSLR's super-high-resolution sensor.

Best results are obtained by using a 4K video camera.
However, the Full HD video resolution offered by most DSLRs in video mode is sufficient for producing images
for screen display or smaller prints.

### Sometimes photos

In some cases, photos may be better suited. Examples are slow subjects,
or when your aim is an image that shows multiple "clones" of the same person,
like in the work of [Daisuke Takakura](https://www.lensculture.com/articles/daisuke-takakura-monodramatic).

Working with photos, however, needs particular care (see next section).

## General considerations

In general, it is of primary importance to have a steady camera position.
Also, a still background with as few motion as possible is desired.
Requirements for both increase as the time span covered increases.
Contrary, with a uniform background like a clear sky, requirements are relaxed.

"Clone photography" is probably the use case which requires the greatest care regarding
a still background and steady camera. 
The long time span covered as well as the higher resolution compared to video take their toll here.
Additionally, some manual corrections may be required afterwards.

As a fallback, `chrono-photo` provides optional camera shake reduction. 
For details, see section [Camera shake reduction](options.md#camera-shake-reduction)
in the options' documentation.

The following points apply no matter whether photos or videos are recorded,
but the focus is primarily on video.

## What you need

* A **camera** or smartphone with video recording capability
* A **tripod**, or another steady base like a bean bag
* Some **time** to spend outside

## Camera settings

### Focus

**Turn auto-focus off!**

Or, even better: set your camera to focus with another button then the trigger button,
or use "hold AF" (or however it is called on your camera).

Focus to where you expect your subject to appear (if not already there).
More exactly: focus to approx. the 1<sup>st</sup> third of the range where your subject may appear
(2/3 behind, 1/3 in front).

See also [Aperture](#aperture).

### Frame rate

Most cameras offer a choice between 30 and 60 frames per second (fps). 
The video resolution may decrease for 60 fps (e.g. only HD instead of Full HD for 30 fps).

Which fps value to choose depends on your subject, and the look you want to achieve.

>If there is no difference in resolution between frame rates on your camera,
choose the maximum possible.
You can still skip frames afterwards using `chrono-photo`, or during preprocessing.

For small and/or fast birds like swifts, but also pigeons and most songbirds,
30 fps will give distinct repetitions of the subject. 
Even with 60 fps, cohesive movement trails may not be possible.

For larger and slower birds, like birds of prey, goose etc.,
cohesive trails are more easily achieved.

For comparison of the effect of frame rates and different speed and size of te subject,
see these images:

<p align="center">
<img src="https://user-images.githubusercontent.com/44003176/79148074-3a65c980-7dc5-11ea-8798-91a817c95600.jpg" alt="Frame rate example with pigeons" width="1024" /><br/>
<i>Different frame rates: A flock of pigeons at 30 fps (left) and 60 fps (right).</i>

<img src="https://user-images.githubusercontent.com/44003176/79621993-c6555980-8115-11ea-9881-3a796933d867.jpg" alt="Frame rate example with pigeons and red kite" width="1024" /><br/>
<i>Different speed and size: Pigeons and a red kite at 30 fps.</i>
</p>

The choice of the frame rate is mostly a trade-off between "as much as possible, skip frames later"
and reduced resolution at higher frame rates.

### Shutter speed

For video recording, most scenarios require far shorter shutter times
than the camera will select by default. 
Human eyes are fairly slow and have a "shutter speed" of around half of their "frame time".
Therefore, we see fast-moving things blurred. 
Cameras try to select a similar shutter speed for video recording to make movement look fluent. 
Typically, standard settings are 1/60s for 30 fps recording, or 1/120s for 60 fps.

For chrono-photography, you most often don't want too much motion blur,
and these shutter speeds are far to slow.

For recording with 30 fps, shutter speeds of 1/250 or faster are recommended (1/500s for 60 fps). 
If light conditions allow, 1/1000s of even faster give very few blur even for fast subjects.
For the example image in section [Frame rate](#frame-rate), 1/800s was used.

Not all cameras support manual selection of shutter speed for video,
and most DSLRs must probably be switched to manual mode.
Determine your preferred combination of shutter speed, aperture and ISO in your preferred photo mode,
then switch to video mode and to manual, and adjust settings accordingly.

### Aperture

Depending on the situation, particularly if you don't know what you will be recording,
it may be advantageous to stop your lens down to achieve a wider depth-of-field (DOF).
This way you can achieve a sharp image even when your subject does not pass in the focus plane.

Of course, the decision is a trade-off between wide depth-of-field and high shutter speed
(see [Shutter speed](#shutter-speed)).

### White balance

In order to avoid color differences between images (or between video frames),
automatic white balancing (AWB) should be turned **off**.
Choose a white balancing preset according the light conditions.

This is of particular importance when taking photos rather than videos
(e.g. "clone photography").

## Summary

We have seen that most points about recording revolve around:
1. a steady camera with constant settings, and
1. a shutter speed fitting the subject as well as the intended result

Finally, the usual considerations for recording photos and videos apply.

----

#### Next tutorial: [Preparing video material](tutorial_prepare.md)