//! Processes time-sliced data produced by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
use crate::color;
use crate::img_stream::{Compression, PixelInputStream};
use crate::{EnumFromString, ParseEnumError};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use rand::{Rng, ThreadRng};
use std::path::PathBuf;
use std::str::FromStr;

/// Pixel selection mode.
#[derive(Debug, Clone)]
pub enum SelectionMode {
    /// Selects by outlier analysis (multi-dimensional z-score).
    /// Parameter `threshold` determines the minimum distance from the median, in fractions of the total color range (i.e. [0, 1]), to classify a pixel as outlier.
    Outlier,
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
            "outlier" => Ok(SelectionMode::Outlier),
            _ => Err(ParseEnumError(format!(
                "Not a pixel selection mode: {}. Must be one of (lighter|darker|outlier)",
                str
            ))),
        }
    }
}

/// Outlier determination algorithm.
#[derive(Debug, Clone)]
pub struct Threshold {
    absolute: bool,
    min: f32,
    max: f32,
    scale: f32,
}
impl Threshold {
    pub fn new(absolute: bool, min: f32, max: f32) -> Self {
        Threshold {
            absolute,
            min,
            max,
            scale: 1.0 / (max - min),
        }
    }
    pub fn blend_value(&self, dist: f32) -> f32 {
        if dist <= self.min {
            0.0
        } else if dist >= self.max {
            1.0
        } else {
            (dist - self.min) * self.scale
        }
    }
}
impl FromStr for Threshold {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        let opt_str = parts
            .get(0)
            .expect(&format!("Unexpected format in {}", str));
        let absolute = match *opt_str {
            "absolute" | "abs" => true,
            "relative" | "rel" => false,
            _ => return Err(ParseEnumError(format!(
                "Not a pixel outlier detection mode: {}. Must be one of (abs[olute]/<threshold>|rel[ative]/<threshold>)",
                str
            ))),
        };

        let thresh_min_str = parts
            .get(1)
            .expect(&format!("Unexpected format in {}", str));
        let mut min = thresh_min_str.parse().expect(&format!(
            "Unable to parse lower threshold for outlier detection: {}",
            str
        ));
        let thresh_max_str = parts.get(2);
        let mut max = match thresh_max_str {
            Some(str) => str.parse().expect(&format!(
                "Unable to parse upper threshold for outlier detection: {}",
                str
            )),
            None => min,
        };
        if absolute {
            min *= 255.0;
            max *= 255.0;
        }

        Ok(Threshold::new(absolute, min, max))
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
    /// Use the median of the pixel from all images (Warning: may result in banding!).
    Median,
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
            "median" => Ok(BackgroundMode::Median),
            _ => Err(ParseEnumError(format!(
                        "Not a background pixel selection mode: {}. Must be one of (first|random|average|median)",
                        str
                    ))),
        }
    }
}

/// Core processor for image analysis.
/// Analysis is based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
pub struct ChronoProcessor {
    mode: SelectionMode,
    threshold: Threshold,
    background: BackgroundMode,
    outlier: OutlierSelectionMode,
    compression: Compression,
    sample: [u8; 4],
    mean: [f32; 4],
    //sd: [f32; 4],
    median: [f32; 4],
    iqr_inv: [f32; 4],
    outlier_indices: Vec<(usize, f32)>,
    values: Vec<u8>,
    rng: ThreadRng,
}

impl ChronoProcessor {
    /// Creates a new image processor.
    pub fn new(
        mode: SelectionMode,
        threshold: Threshold,
        bg_mode: BackgroundMode,
        outlier_mode: OutlierSelectionMode,
        compression: Compression,
    ) -> Self {
        ChronoProcessor {
            mode,
            threshold,
            background: bg_mode,
            outlier: outlier_mode,
            compression,
            sample: [0; 4],
            mean: [0.0; 4],
            median: [0.0; 4],
            iqr_inv: [0.0; 4],
            //sd: [0.0; 4],
            outlier_indices: vec![],
            values: vec![],
            rng: rand::thread_rng(),
        }
    }
    /// Processes images based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
    pub fn process(
        mut self,
        layout: &SampleLayout,
        files: &[PathBuf],
        size_hint: Option<usize>,
    ) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
        let channels = layout.width_stride;
        let mut buffer = vec![0; layout.height as usize * layout.height_stride];
        let mut is_outlier = vec![0; layout.height as usize * layout.height_stride];

        let mut pixel_data = Vec::new();
        let mut pixel = vec![0; channels];

        println!("Processing {} time slices", files.len());
        let bar = ProgressBar::new(files.len() as u64);
        for (out_row, file) in files.iter().enumerate() {
            bar.inc(1);

            let buff_row_start = out_row * layout.height_stride;
            let mut data = match size_hint {
                Some(hint) => Vec::with_capacity(hint * layout.height as usize),
                None => Vec::new(),
            };
            let mut num_rows = 0;
            {
                let mut stream = PixelInputStream::new(file, self.compression.clone())?;
                while let Some(_num_bytes) = stream.read_chunk(&mut data) {
                    num_rows += 1;
                }
            }
            if pixel_data.len() != num_rows * channels {
                pixel_data = vec![0; num_rows * channels];
            }
            if self.outlier_indices.len() != num_rows {
                self.outlier_indices = vec![(0, 0.0); num_rows];
                self.values = vec![0; num_rows * channels];
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
                let blend = self.calc_pixel(&pixel_data, &mut pixel);
                for ch in 0..channels {
                    let idx = buff_row_start + col as usize * channels + ch;
                    buffer[idx] = pixel[ch];
                    if ch < 3 {
                        is_outlier[idx] = blend;
                    } else {
                        is_outlier[idx] = 255;
                    }
                }
            }
        }
        bar.finish_and_clear();

        Ok((buffer, is_outlier))
    }

    fn calc_pixel(&mut self, pixel_data: &[u8], pixel: &mut [u8]) -> u8 {
        let mode = &self.mode.clone(); // TODO find a way to avoid clone
        match mode {
            SelectionMode::Darker => self.calc_pixel_darker(pixel_data, pixel),
            SelectionMode::Lighter => self.calc_pixel_lighter(pixel_data, pixel),
            SelectionMode::Outlier => {
                self.calc_pixel_z_score(pixel_data, pixel, self.threshold.clone())
            }
        }
    }

    fn calc_pixel_darker(&self, pixel_data: &[u8], pixel: &mut [u8]) -> u8 {
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
        0
    }
    fn calc_pixel_lighter(&self, pixel_data: &[u8], pixel: &mut [u8]) -> u8 {
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
        0
    }

    fn calc_pixel_z_score(
        &mut self,
        pixel_data: &[u8],
        mut pixel: &mut [u8],
        threshold: Threshold,
    ) -> u8 {
        let channels = pixel.len();
        let samples = pixel_data.len() / channels;

        let threshold_sq = threshold.min * threshold.min;

        // Reset mean and SD
        for m in self.mean.iter_mut() {
            *m = 0.0;
        }

        // Calculate mean of samples, prepare medians
        for (sample_idx, pix) in pixel_data.chunks(channels).enumerate() {
            for (i, p) in pix.iter().enumerate() {
                self.mean[i] += *p as f32;
                self.values[i * samples + sample_idx] = *p;
            }
        }
        for m in self.mean.iter_mut() {
            *m /= samples as f32;
        }
        // Calculate medians and inverse inter-quartile range
        for i in 0..channels {
            let slice = &mut self.values[(i * samples)..(i * samples + samples)];
            slice.sort_unstable();
            if threshold.absolute {
                self.median[i] = Self::median(slice);
            } else {
                let (q1, med, q3) = Self::quartiles(slice);
                self.median[i] = med;
                self.iqr_inv[i] = q3 - q1;
                if self.iqr_inv[i] == 0.0 {
                    self.iqr_inv[i] = 1.0;
                }
                self.iqr_inv[i] = 1.0 / self.iqr_inv[i];
            }
        }

        let mut num_outliers = 0;
        let mut max_dist_sq = 0.0;
        let mut max_index = 0;

        for (sample_idx, pix) in pixel_data.chunks(channels).enumerate() {
            let mut dist_sq = 0.0;
            for (i, p) in pix.iter().enumerate() {
                let diff = self.median[i] - *p as f32;
                dist_sq += if diff == 0.0 {
                    0.0
                } else {
                    if threshold.absolute {
                        (diff * diff)
                    } else {
                        (self.iqr_inv[i] * diff).powi(2)
                    }
                }
            }
            if dist_sq >= threshold_sq {
                self.outlier_indices[num_outliers] = (sample_idx, dist_sq);
                num_outliers += 1;
                if dist_sq > max_dist_sq {
                    max_dist_sq = dist_sq;
                    max_index = sample_idx;
                }
            }
        }

        // Fill pixel with background
        match self.background {
            BackgroundMode::Average => {
                for ch in 0..channels {
                    pixel[ch] = self.mean[ch].round() as u8;
                }
            }
            BackgroundMode::Median => {
                for ch in 0..channels {
                    pixel[ch] = self.median[ch].round() as u8;
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

        let has_outliers = num_outliers > 0;
        if has_outliers {
            // Get outlier
            let (sample, dist) = if self.outlier == OutlierSelectionMode::Average {
                if num_outliers == 1 {
                    let (sample_idx, dist_sq) = self.outlier_indices[0];
                    let sample =
                        &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];
                    (sample, dist_sq.sqrt())
                } else {
                    for ch in 0..channels {
                        self.mean[ch] = 0.0;
                    }
                    let mut mean_dist = 0.0;
                    for (sample_idx, dist_sq) in self.outlier_indices.iter().take(num_outliers) {
                        let offset = sample_idx * channels;
                        for ch in 0..channels {
                            self.mean[ch] += pixel_data[offset + ch] as f32;
                        }
                        mean_dist += dist_sq.sqrt();
                    }
                    for ch in 0..channels {
                        self.sample[ch] = (self.mean[ch] / num_outliers as f32).round() as u8;
                    }
                    /*for ch in 0..channels {
                        pixel[ch] = (self.mean[ch] / num_outliers as f32).round() as u8;
                    }*/
                    (&self.sample[..], mean_dist / num_outliers as f32)
                }
            } else {
                let (sample_idx, dist_sq) = match self.outlier {
                    OutlierSelectionMode::First => self.outlier_indices[0],
                    OutlierSelectionMode::Last => self.outlier_indices[num_outliers - 1],
                    OutlierSelectionMode::Extreme => (max_index, max_dist_sq),
                    OutlierSelectionMode::Average => (0, 0.0),
                };
                let sample =
                    &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];
                (sample, dist_sq.sqrt())
            };
            // Blend outlier into background
            let blend = threshold.blend_value(dist);
            color::blend_into(&mut pixel, &sample, blend);
            (blend * 255.0).round() as u8
        } else {
            0
        }
    }

    /// Calculates quartiles from a sample (approximated).
    ///
    /// Return (Q1, Median, Q3)
    fn quartiles(data: &[u8]) -> (f32, f32, f32) {
        let len = data.len();

        let med = if len % 2 == 0 {
            data[(len + 1) / 2] as f32
        } else {
            let idx = len / 2;
            0.5 * (data[idx] as f32 + data[idx + 1] as f32)
        };

        let q1 = if (len + 1) % 4 == 0 {
            data[(len + 1) / 4] as f32
        } else {
            let idx = len / 4;
            0.5 * (data[idx] as f32 + data[idx + 1] as f32)
        };

        let q3 = if (3 * (len + 1)) % 4 == 0 {
            data[(3 * (len + 1)) / 4] as f32
        } else {
            let idx = (3 * len) / 4;
            0.5 * (data[idx] as f32 + data[idx + 1] as f32)
        };
        (q1, med, q3)
    }

    /// Calculates the median of a sample.
    fn median(data: &[u8]) -> f32 {
        let len = data.len();

        if len % 2 == 0 {
            data[(len + 1) / 2] as f32
        } else {
            let idx = len / 2;
            0.5 * (data[idx] as f32 + data[idx + 1] as f32)
        }
    }
}
