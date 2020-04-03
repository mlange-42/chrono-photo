//! Converts a series of images by time to images by row. I.e. transposes (x,y) in the cube in (x,y,t) to (x,t).

use crate::img_stream::{ImageStream, PixelOutputStream};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use std::fmt;
use std::path::PathBuf;

/// Converts a series of images by time to images by row. I.e. transposes (x,y) in the cube in (x,y,t) to (x,t).
pub struct TimeSlicer();

impl TimeSlicer {
    /// Writes time slices for all images in the given stream, into the given temporary directory.
    /// Files are named `temp-xxxxx.gz`.
    pub fn write_time_slices(
        images: ImageStream,
        temp_dir: PathBuf,
    ) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
        assert!(temp_dir.is_dir());
        let size_hint = images.len();

        let mut layout: Option<SampleLayout> = None;
        let mut count = 0;

        let mut out_streams: Option<Vec<PixelOutputStream>> = None;

        println!("Time-slicing {} images", size_hint);
        let bar = ProgressBar::new(size_hint as u64);
        for img in images {
            bar.inc(1);

            let dyn_img = img.unwrap();
            let pix = dyn_img.as_flat_samples_u8().unwrap();
            let lay = match layout {
                Some(lay) => {
                    if pix.layout != lay {
                        return Err(TimeSliceError("Image layout does not fit!".to_string()));
                    }
                    lay
                }
                None => {
                    layout = Some(pix.layout);
                    pix.layout
                }
            };
            // TODO: Change to one file per X rows instead of one per row?
            // E.g. 4K would produce 2160 files.
            if out_streams.is_none() {
                out_streams = Some(
                    (0..lay.height)
                        .map(|i| {
                            let mut path = PathBuf::from(&temp_dir);
                            path.push(format!("temp-{:05}.gz", i));
                            PixelOutputStream::new(&path)
                                .expect(&format!("Unable to create file {:?}", path))
                        })
                        .collect(),
                );
            }
            let stride = lay.height_stride;
            for (row, stream) in out_streams.as_mut().unwrap().iter_mut().enumerate() {
                let start = row * stride;
                let end = (row + 1) * stride;
                stream
                    .write_chunk(&pix.samples[start..end])
                    .expect(&format!(
                        "Unable to write chunk to file {:?}",
                        stream.path()
                    ));
            }
            count += 1;
        }

        for stream in out_streams.as_mut().unwrap().iter_mut() {
            stream.close().unwrap();
        }

        if count == 0 {
            Err(TimeSliceError("No images found for pattern {}".to_string()))
        } else {
            Ok((
                out_streams
                    .unwrap()
                    .iter()
                    .map(|stream| stream.path().clone())
                    .collect(),
                layout.unwrap(),
                size_hint,
            ))
        }
    }
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSliceError(String);

impl fmt::Display for TimeSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
