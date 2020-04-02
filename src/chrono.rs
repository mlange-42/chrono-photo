//! Processes time-sliced data produced by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
use crate::img_stream::PixelInputStream;
use crate::{EnumFromString, ParseEnumError};
use image::flat::SampleLayout;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SelectionMode {
    Outlier,
    Lighter,
    Darker,
}
impl EnumFromString for SelectionMode {
    fn from_string(str: &str) -> Result<Self, ParseEnumError>
    where
        Self: std::marker::Sized,
    {
        match str {
            "lighter" => Ok(SelectionMode::Lighter),
            "darker" => Ok(SelectionMode::Darker),
            "outlier" => Ok(SelectionMode::Outlier),
            _ => Err(ParseEnumError(format!(
                "Not a pixel selection mode: {}. Must be one of (lighter|darker|outlier)",
                str
            ))),
        }
    }
}

pub struct ChronoProcessor {
    mode: SelectionMode,
}

impl ChronoProcessor {
    pub fn new(mode: SelectionMode) -> Self {
        ChronoProcessor { mode }
    }

    pub fn process(
        &self,
        layout: &SampleLayout,
        files: &[PathBuf],
        size_hint: Option<usize>,
    ) -> std::io::Result<Vec<u8>> {
        let channels = layout.width_stride;
        let mut buffer = vec![0; layout.height as usize * layout.height_stride];

        let mut pixel_data = Vec::new();
        let mut pixel = vec![0; channels];

        for (out_row, file) in files.iter().enumerate() {
            let buff_row_start = out_row * layout.height_stride;
            let mut stream = PixelInputStream::new(file)?;
            let mut data = match size_hint {
                Some(hint) => Vec::with_capacity(hint * layout.height as usize),
                None => Vec::new(),
            };
            let mut num_rows = 0;
            while let Some(_num_bytes) = stream.read_chunk(&mut data) {
                num_rows += 1;
            }
            if pixel_data.is_empty() {
                pixel_data = vec![0; num_rows * channels];
            }
            for col in 0..layout.width {
                let col_offset = col as usize * channels;
                for row in 0..num_rows {
                    let pix_start = row * layout.height_stride + col_offset;
                    for ch in 0..channels {
                        let v = data[pix_start + ch];
                        pixel_data[row * channels + ch] = v;
                    }
                }
                self.calc_pixel(&pixel_data, &mut pixel);
                for ch in 0..channels {
                    buffer[buff_row_start + col as usize * channels + ch] = pixel[ch];
                }
            }
        }

        Ok(buffer)
    }

    fn calc_pixel(&self, pixel_data: &[u8], pixel: &mut [u8]) {
        match &self.mode {
            SelectionMode::Darker => self.calc_pixel_darker(pixel_data, pixel),
            SelectionMode::Lighter => self.calc_pixel_lighter(pixel_data, pixel),
            SelectionMode::Outlier => unimplemented!(),
        }
    }

    fn calc_pixel_darker(&self, pixel_data: &[u8], pixel: &mut [u8]) {
        let channels = pixel.len();
        let pixels = pixel_data.len() / channels;

        let mut sum_min = std::u32::MAX;
        let mut idx_min = 0;
        for pix in 0..pixels {
            let idx = pix * channels;
            let sum = pixel_data[idx..(idx + 3)].iter().map(|v| *v as u32).sum();
            if sum < sum_min {
                sum_min = sum;
                idx_min = pix;
            }
        }
        for ch in 0..channels {
            pixel[ch] = pixel_data[idx_min * channels + ch];
        }
    }
    fn calc_pixel_lighter(&self, pixel_data: &[u8], pixel: &mut [u8]) {
        let channels = pixel.len();
        let pixels = pixel_data.len() / channels;

        let mut sum_max = std::u32::MIN;
        let mut idx_max = 0;
        for pix in 0..pixels {
            let idx = pix * channels;
            let sum = pixel_data[idx..(idx + 3)].iter().map(|v| *v as u32).sum();
            if sum > sum_max {
                sum_max = sum;
                idx_max = pix;
            }
        }
        for ch in 0..channels {
            pixel[ch] = pixel_data[idx_max * channels + ch];
        }
    }
}
