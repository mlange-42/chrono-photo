//! Converts a series of images by time to images by row. I.e. transposes (x,y) in the cube in (x,y,t) to (x,t).

use crate::shake::Crop;
use crate::slicer::SliceLength::{Count, Pixels, Rows};
use crate::streams::{Compression, ImageStream, PixelOutputStream};
use crate::ParseEnumError;
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use num_traits::PrimInt;
use rand::Rng;
use rayon::prelude::*;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

const HEX_CHARS: &str = "0123456789abcdef";

#[derive(Debug, Clone)]
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
#[allow(dead_code)]
pub struct TimeSlicer<T>
where
    T: PrimInt,
{
    is_16: bool,
    dummy: T,
}

impl TimeSlicer<u8> {
    pub fn new_8bit() -> Self {
        TimeSlicer {
            is_16: false,
            dummy: 0_u8,
        }
    }
}
impl TimeSlicer<u16> {
    pub fn new_16bit() -> Self {
        TimeSlicer {
            is_16: true,
            dummy: 0_u16,
        }
    }
}

impl<T> TimeSlicer<T>
where
    T: PrimInt,
{
    /// Writes time slices for all images in the given stream, into the given temporary directory.
    /// Files are named `temp-xxxxx.bin`.
    pub fn write_time_slices(
        &self,
        images: ImageStream,
        crop: &Option<Vec<Crop>>,
        temp_dir: PathBuf,
        compression: &Compression,
        slices: &SliceLength,
    ) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
        assert!(temp_dir.is_dir());
        let size_hint = images.len();

        let mut rng = rand::thread_rng();
        let chars: Vec<char> = HEX_CHARS.chars().collect();
        let id: String = (0..12)
            .map(|_| chars[rng.gen_range(0, chars.len())])
            .collect();

        let mut layout: Option<SampleLayout> = None;
        let mut slicing: Option<(usize, usize)> = None;
        let mut count = 0;

        let mut files: Option<Vec<(usize, PathBuf)>> = None;

        let mut total_bytes: u32 = 0;
        let mut total_files = 0;
        println!("Time-slicing {} images", size_hint);
        let bar = ProgressBar::new(size_hint as u64);
        bar.set_draw_delta((size_hint / 200) as u64);
        for (img_index, img) in images.enumerate() {
            bar.inc(1);

            let mut dyn_img = img.unwrap();
            if let Some(crop) = crop {
                dyn_img = crop[img_index].crop(&mut dyn_img);
            }
            let pix = dyn_img
                .as_flat_samples_u8()
                .expect("Unexpected format. Not an 8 bit image.");

            let lay = match layout {
                Some(lay) => {
                    //println!("{:?} vs. {:?}", pix.layout, lay);
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
            total_files = slice_count;

            if files.is_none() {
                files = Some(
                    (0..slice_count)
                        .map(|i| {
                            let mut path = PathBuf::from(&temp_dir);
                            path.push(format!("temp-{}-{:05}.bin", id, i));
                            (i, path)
                        })
                        .collect(),
                );
            }

            let stride = slice_bytes;
            let num_sample = pix.samples.len();
            total_bytes += files
                .as_ref()
                .unwrap()
                .par_iter()
                .map(|(row, path)| {
                    let start = row * stride;
                    let end = std::cmp::min((row + 1) * stride, num_sample);

                    let mut stream =
                        PixelOutputStream::new(&path, compression.clone(), img_index > 0)
                            .expect(&format!("Unable to create file {:?}", path));

                    stream
                        .write_chunk(&pix.samples[start..end])
                        .expect(&format!(
                            "Unable to write chunk to file {:?}",
                            stream.path()
                        ))
                })
                .sum::<usize>() as u32;
            count += 1;
        }
        bar.finish_and_clear();
        println!("Total: {} kb in {} files", total_bytes / 1024, total_files);

        /*for stream in out_streams.as_mut().unwrap().iter_mut() {
            stream.close().unwrap();
        }*/

        if count == 0 {
            Err(TimeSliceError(
                "No images found for given pattern".to_string(),
            ))
        } else {
            Ok((
                (0..total_files)
                    .map(|i| {
                        let mut path = PathBuf::from(&temp_dir);
                        path.push(format!("temp-{}-{:05}.bin", id, i));
                        path
                    })
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
