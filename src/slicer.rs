//! Converts a series of images by time to images by row. I.e. transposes (x,y) in the cube in (x,y,t) to (x,t).

use crate::slicer::SliceLength::{Count, Pixels, Rows};
use crate::streams::{Compression, ImageStream, PixelOutputStream};
use crate::ParseEnumError;
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub enum SliceLength {
    Rows(usize),
    Pixels(usize),
    Count(usize),
}
impl SliceLength {
    pub fn bytes(&self, layout: &SampleLayout) -> usize {
        match self {
            Pixels(n) => *n * layout.width_stride,
            Count(n) => {
                ((layout.height_stride as u32 * layout.height) as f32 / *n as f32).ceil() as usize
            }
            Rows(n) => *n * layout.height_stride,
        }
    }
    pub fn count(&self, layout: &SampleLayout) -> usize {
        match self {
            Pixels(n) => ((layout.height * layout.width) as f32 / *n as f32).ceil() as usize,
            Count(n) => *n,
            Rows(n) => (layout.height as f32 / *n as f32).ceil() as usize,
        }
    }
}
impl FromStr for SliceLength {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        let opt_str = parts
            .get(0)
            .expect(&format!("Unexpected format in {}.", str));

        let number = parts
            .get(1)
            .expect(&format!("Unexpected format in {}", str));
        let value = number
            .parse()
            .expect(&format!("Unable to parse slicing numeric part: {}", str));

        let s = match opt_str {
            &"rows" => Rows(value),
            &"pixels" => Pixels(value),
            &"count" => Count(value),
            _ => {
                return Err(ParseEnumError(format!(
                    "Not a valid slicing mode: {}. Must be one of (rows|pixels|count)/<number>",
                    str
                )))
            }
        };
        Ok(s)
    }
}

/// Converts a series of images by time to images by row. I.e. transposes (x,y) in the cube in (x,y,t) to (x,t).
pub struct TimeSlicer();

impl TimeSlicer {
    /// Writes time slices for all images in the given stream, into the given temporary directory.
    /// Files are named `temp-xxxxx.bin`.
    pub fn write_time_slices(
        images: ImageStream,
        temp_dir: PathBuf,
        compression: &Compression,
        slices: &SliceLength,
    ) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
        assert!(temp_dir.is_dir());
        let size_hint = images.len();

        let mut layout: Option<SampleLayout> = None;
        let mut slicing: Option<(usize, usize)> = None;
        let mut count = 0;

        let mut out_streams: Option<Vec<PixelOutputStream>> = None;

        let mut total_bytes = 0;
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

            let sli = match slicing {
                Some(sl) => sl,
                None => {
                    let sl = (slices.bytes(&lay), slices.count(&lay));
                    slicing = Some(sl);
                    sl
                }
            };
            let slice_bytes = sli.0;
            let slice_count = sli.1;

            if out_streams.is_none() {
                out_streams = Some(
                    (0..slice_count)
                        .map(|i| {
                            let mut path = PathBuf::from(&temp_dir);
                            path.push(format!("temp-{:05}.bin", i));
                            PixelOutputStream::new(&path, compression.clone())
                                .expect(&format!("Unable to create file {:?}", path))
                        })
                        .collect(),
                );
            }
            let stride = slice_bytes; //lay.height_stride;
            let num_sample = pix.samples.len();
            for (row, stream) in out_streams.as_mut().unwrap().iter_mut().enumerate() {
                let start = row * stride;
                let end = std::cmp::min((row + 1) * stride, num_sample);
                //println!("{} - {}, ({}) by {}", start, end, pix.samples.len(), stride);
                total_bytes += stream
                    .write_chunk(&pix.samples[start..end])
                    .expect(&format!(
                        "Unable to write chunk to file {:?}",
                        stream.path()
                    ));
            }
            count += 1;
        }
        bar.finish_and_clear();
        println!(
            "Total: {} kb in {} files",
            total_bytes / 1024,
            out_streams
                .as_mut()
                .expect("No input images supplied!")
                .len()
        );

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

/// Error type for failed time-slicing due to wrong data layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSliceError(String);

impl fmt::Display for TimeSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
