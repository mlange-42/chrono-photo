# Recording material for chrono-photo

Next tutorial: [Creating chrono-photos](tutorial_photos.md).

This tutorial covers how to record material for use with `chrono-photo`,
particularly for fast-moving subjects like birds.

Fast subjects require recording videos rather than photos due to the frame rate limitations
when using a camera in still photography mode. 
So you will unfortunately not be able to take advantage of your DSLR's super-high-resolution sensor.

Best results are obtained by using a 4K video camera. 
However, the Full HD video resolution offered by most DSLRs is sufficient for producing images
for screen display or smaller prints.

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

For recording with 30 fps, shutter speeds of 1/250 or faster are recommended (1/500 for 60 fps). 
If light conditions allow, 1/1000 of even faster give very few blur even for fast subjects.

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

#### To next tutorial: [Creating chrono-photos](tutorial_photos.md)