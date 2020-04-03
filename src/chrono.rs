//! Processes time-sliced data produced by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
use crate::img_stream::{Compression, PixelInputStream};
use crate::{EnumFromString, ParseEnumError};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use rand::{Rng, ThreadRng};
use std::path::PathBuf;

/// Pixel selection mode.
#[derive(Debug, Clone)]
pub enum SelectionMode {
    /// Selects by outlier analysis (multi-dimensional z-score).
    /// Parameter `threshold` determines the minimum distance from the mean, in multiples of standard deviation, to classify a pixel as outlier.
    Outlier { threshold: f32 },
    /// Selects the lightest/brightest pixel (sum of red, green and blue).
    Lighter,
    /// Selects the darkest pixel (sum of red, green and blue).
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
            _ => {
                if str.starts_with("outlier") {
                    let str = str
                        .split('-')
                        .nth(1)
                        .expect(&format!("Unexpected format in {}", str));
                    let v = str.parse().expect(&format!(
                        "Unable to parse threshold for outlier detection: {}",
                        str
                    ));
                    Ok(SelectionMode::Outlier { threshold: v })
                } else {
                    Err(ParseEnumError(format!(
                        "Not a pixel selection mode: {}. Must be one of (lighter|darker|outlier-<threshold>)",
                        str
                    )))
                }
            }
        }
    }
}

/// Selection mode if multiple outliers are found.
#[derive(Debug, Clone, PartialEq)]
pub enum OutlierSelectionMode {
    /// Use the first outlier.
    First,
    /// Use the last outlier.
    Last,
    /// Use the most extreme outlier.
    Extreme,
    /// Use the most average of all outlier.
    Average,
}
impl EnumFromString for OutlierSelectionMode {
    fn from_string(str: &str) -> Result<Self, ParseEnumError>
    where
        Self: std::marker::Sized,
    {
        match str {
            "first" => Ok(OutlierSelectionMode::First),
            "last" => Ok(OutlierSelectionMode::Last),
            "extreme" => Ok(OutlierSelectionMode::Extreme),
            "average" => Ok(OutlierSelectionMode::Average),
            _ => Err(ParseEnumError(format!(
                "Not an outlier selection mode: {}. Must be one of (first|last|extreme|average)",
                str
            ))),
        }
    }
}

/// Background pixel selection mode, i.e. when no outliers are found.
#[derive(Debug, Clone)]
pub enum BackgroundMode {
    /// Use the pixel from the first image.
    First,
    /// Use the pixel from a randomly selected image.
    Random,
    /// Use the average of the pixel from all images (Warning: may result in banding!).
    Average,
}
impl EnumFromString for BackgroundMode {
    fn from_string(str: &str) -> Result<Self, ParseEnumError>
    where
        Self: std::marker::Sized,
    {
        match str {
            "first" => Ok(BackgroundMode::First),
            "random" => Ok(BackgroundMode::Random),
            "average" => Ok(BackgroundMode::Average),
            _ => Err(ParseEnumError(format!(
                        "Not a background pixel selection mode: {}. Must be one of (lighter|darker|outlier-<threshold>)",
                        str
                    ))),
        }
    }
}

/// Core processor for image analysis.
/// Analysis is based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
pub struct ChronoProcessor {
    mode: SelectionMode,
    background: BackgroundMode,
    outlier: OutlierSelectionMode,
    compression: Compression,
    mean: [f32; 4],
    sd: [f32; 4],
    outlier_indices: Vec<usize>,
    rng: ThreadRng,
}

impl ChronoProcessor {
    /// Creates a new image processor.
    pub fn new(
        mode: SelectionMode,
        bg_mode: BackgroundMode,
        outlier_mode: OutlierSelectionMode,
        compression: Compression,
    ) -> Self {
        ChronoProcessor {
            mode,
            background: bg_mode,
            outlier: outlier_mode,
            compression,
            mean: [0.0; 4],
            sd: [0.0; 4],
            outlier_indices: vec![],
            rng: rand::thread_rng(),
        }
    }
    /// Processes images based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
    pub fn process(
        mut self,
        layout: &SampleLayout,
        files: &[PathBuf],
        size_hint: Option<usize>,
    ) -> std::io::Result<Vec<u8>> {
        let channels = layout.width_stride;
        let mut buffer = vec![0; layout.height as usize * layout.height_stride];

        let mut pixel_data = Vec::new();
        let mut pixel = vec![0; channels];

        println!("Processing {} time slices", files.len());
        let bar = ProgressBar::new(files.len() as u64);
        for (out_row, file) in files.iter().enumerate() {
            bar.inc(1);

            let buff_row_start = out_row * layout.height_stride;
            let mut stream = PixelInputStream::new(file, self.compression.clone())?;
            let mut data = match size_hint {
                Some(hint) => Vec::with_capacity(hint * layout.height as usize),
                None => Vec::new(),
            };
            let mut num_rows = 0;
            while let Some(_num_bytes) = stream.read_chunk(&mut data) {
                num_rows += 1;
            }
            if pixel_data.len() != num_rows * channels {
                pixel_data = vec![0; num_rows * channels];
            }
            if self.outlier_indices.len() != num_rows {
                self.outlier_indices = vec![0; num_rows];
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

    fn calc_pixel(&mut self, pixel_data: &[u8], pixel: &mut [u8]) {
        let mode = &self.mode.clone(); // TODO find a way to avoid clone
        match mode {
            SelectionMode::Darker => self.calc_pixel_darker(pixel_data, pixel),
            SelectionMode::Lighter => self.calc_pixel_lighter(pixel_data, pixel),
            SelectionMode::Outlier { threshold: thresh } => {
                self.calc_pixel_z_score(pixel_data, pixel, *thresh)
            }
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

    fn calc_pixel_z_score(&mut self, pixel_data: &[u8], pixel: &mut [u8], threshold: f32) {
        let channels = pixel.len();
        let samples = pixel_data.len() / channels;
        for m in self.mean.iter_mut() {
            *m = 0.0;
        }
        // Calculate mean of samples
        for pix in pixel_data.chunks(channels) {
            for (i, p) in pix.iter().enumerate() {
                self.mean[i] += *p as f32;
            }
        }
        for m in self.mean.iter_mut() {
            *m /= samples as f32;
        }
        // Calculate SD of samples
        for pix in pixel_data.chunks(channels) {
            for (i, p) in pix.iter().enumerate() {
                self.sd[i] += (*p as f32 - self.mean[i]).powi(2);
            }
        }
        for sd in self.sd.iter_mut() {
            *sd = 1.0 / (*sd / (samples - 1) as f32).sqrt();
        }

        let threshold_sq = threshold * threshold;
        let mut num_outliers = 0;
        let mut max_dist_sq = 0.0;
        let mut max_index = 0;
        for (sample_idx, pix) in pixel_data.chunks(channels).enumerate() {
            let mut dist_sq = 0.0;
            for (i, p) in pix.iter().enumerate() {
                let diff = self.mean[i] - *p as f32;
                dist_sq += if diff == 0.0 {
                    0.0
                } else {
                    (self.sd[i] * diff).powi(2)
                }
            }
            if dist_sq > threshold_sq {
                self.outlier_indices[num_outliers] = sample_idx;
                num_outliers += 1;
                if dist_sq > max_dist_sq {
                    max_dist_sq = dist_sq;
                    max_index = sample_idx;
                }
            }
        }
        let has_outliers = num_outliers > 0;
        if has_outliers {
            if self.outlier == OutlierSelectionMode::Average {
                if num_outliers == 1 {
                    let sample_idx = self.outlier_indices[0];
                    let sample =
                        &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];
                    for ch in 0..channels {
                        pixel[ch] = sample[ch];
                    }
                } else {
                    for ch in 0..channels {
                        self.mean[ch] = 0.0;
                    }
                    for sample_idx in self.outlier_indices.iter().take(num_outliers) {
                        let offset = sample_idx * channels;
                        for ch in 0..channels {
                            self.mean[ch] += pixel_data[offset + ch] as f32;
                        }
                    }
                    for ch in 0..channels {
                        pixel[ch] = (self.mean[ch] / num_outliers as f32).round() as u8;
                    }
                }
            } else {
                let sample_idx = match self.outlier {
                    OutlierSelectionMode::First => self.outlier_indices[0],
                    OutlierSelectionMode::Last => self.outlier_indices[num_outliers - 1],
                    OutlierSelectionMode::Extreme => max_index,
                    OutlierSelectionMode::Average => 0,
                };
                let sample =
                    &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];
                for ch in 0..channels {
                    pixel[ch] = sample[ch];
                }
            }
        } else {
            match self.background {
                BackgroundMode::Average => {
                    for ch in 0..channels {
                        pixel[ch] = self.mean[ch].round() as u8;
                    }
                }
                BackgroundMode::First | BackgroundMode::Random => {
                    let sample_idx = match self.background {
                        BackgroundMode::First => 0,
                        BackgroundMode::Random => self.rng.gen_range(0, samples),
                        _ => 0,
                    };
                    let sample =
                        &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];

                    for ch in 0..channels {
                        pixel[ch] = sample[ch];
                    }
                }
            }
        }
    }
}
