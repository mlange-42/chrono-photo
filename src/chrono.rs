//! Processes time-sliced data produced by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
use crate::color;
use crate::options::{BackgroundMode, OutlierSelectionMode, SelectionMode, Threshold};
use crate::slicer::SliceLength;
use crate::streams::{Compression, PixelInputStream};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use rand::{Rng, ThreadRng};
use std::fmt;
use std::path::PathBuf;

/// Error type for failed selection of background pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelSelectionError(String);

impl fmt::Display for PixelSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Per-thread data structures to avoid vector allocations.
struct ThreadData {
    outlier_indices: Vec<(usize, f32)>,
    non_outlier_indices: Vec<usize>,
    values: Vec<u8>,
    rng: ThreadRng,
}

/// Core processor for image analysis.
/// Analysis is based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
pub struct ChronoProcessor {
    mode: SelectionMode,
    threshold: Threshold,
    background: BackgroundMode,
    outlier: OutlierSelectionMode,
    compression: Compression,
    data: ThreadData,
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
            data: ThreadData {
                outlier_indices: vec![],
                non_outlier_indices: vec![],
                values: vec![],
                rng: rand::thread_rng(),
            },
        }
    }
    /// Processes images based on files as created by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
    pub fn process(
        mut self,
        layout: &SampleLayout,
        files: &[PathBuf],
        slices: &SliceLength,
        size_hint: Option<usize>,
    ) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
        let channels = layout.width_stride;
        let mut buffer = vec![0; layout.height as usize * layout.height_stride];
        let mut is_outlier = vec![0; layout.height as usize * layout.height_stride];

        let mut pixel_data = Vec::new();
        let mut pixel = vec![0; channels];

        let mut warnings = 0;
        let slice_bytes = slices.bytes(&layout);
        //let slice_count = slices.count(&layout);

        println!("Processing {} time slices", files.len());
        let bar = ProgressBar::new(files.len() as u64);
        for (out_row, file) in files.iter().enumerate() {
            bar.inc(1);

            let buff_row_start = out_row * slice_bytes; //layout.height_stride;
            let mut data = match size_hint {
                //Some(hint) => Vec::with_capacity(hint * layout.height as usize),
                Some(hint) => Vec::with_capacity(hint * slice_bytes),
                None => Vec::new(),
            };
            let mut num_rows = 0;
            let mut num_bytes = 0;
            {
                let mut stream = PixelInputStream::new(file, self.compression.clone())?;
                while let Some(n_bytes) = stream.read_chunk(&mut data) {
                    num_rows += 1;
                    if num_bytes == 0 {
                        num_bytes = n_bytes;
                    } else if num_bytes != n_bytes {
                        panic!("Unexpected data alignment in slice file {:?}", file);
                    }
                }
            }
            if pixel_data.len() != num_rows * channels {
                pixel_data = vec![0; num_rows * channels];
            }
            if self.data.outlier_indices.len() != num_rows {
                self.data.outlier_indices = vec![(0, 0.0); num_rows];
                self.data.non_outlier_indices = vec![0; num_rows];
                self.data.values = vec![0; num_rows * channels];
            }
            //for col in 0..layout.width {
            for col in 0..(num_bytes / channels) {
                let col_offset = col as usize * channels;
                for row in 0..num_rows {
                    //let pix_start = row * layout.height_stride + col_offset;
                    let pix_start = row * num_bytes + col_offset;
                    for ch in 0..channels {
                        let v = data[pix_start + ch];
                        pixel_data[row * channels + ch] = v;
                    }
                }

                let pix_offset = buff_row_start + col as usize * channels;

                /*let coord = (
                    (pix_offset % layout.height_stride) as usize / channels,
                    (pix_offset / layout.height_stride) as usize,
                );*/

                let (blend, warning) = self.calc_pixel(&pixel_data, &mut pixel);
                if warning {
                    warnings += 1;
                }
                for ch in 0..channels {
                    let idx = pix_offset + ch;
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

        if warnings > 0 {
            println!(
                "Warning: {:?} pixels seem to consist of only outliers",
                warnings
            );
        }

        Ok((buffer, is_outlier))
    }

    fn calc_pixel(&mut self, pixel_data: &[u8], pixel: &mut [u8]) -> (u8, bool) {
        let mode = &self.mode.clone(); // TODO find a way to avoid clone
        match mode {
            SelectionMode::Darker => self.calc_pixel_darker(pixel_data, pixel),
            SelectionMode::Lighter => self.calc_pixel_lighter(pixel_data, pixel),
            SelectionMode::Outlier => {
                self.calc_pixel_z_score(pixel_data, pixel, self.threshold.clone())
            }
        }
    }

    fn calc_pixel_darker(&self, pixel_data: &[u8], pixel: &mut [u8]) -> (u8, bool) {
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
        (0, false)
    }
    fn calc_pixel_lighter(&self, pixel_data: &[u8], pixel: &mut [u8]) -> (u8, bool) {
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
        (0, false)
    }

    fn calc_pixel_z_score(
        &mut self,
        pixel_data: &[u8],
        mut pixel: &mut [u8],
        threshold: Threshold,
    ) -> (u8, bool) {
        let channels = pixel.len();
        let samples = pixel_data.len() / channels;

        let threshold_sq = threshold.min() * threshold.min();

        let mut median = [0.0; 4];
        let mut iqr_inv = [0.0; 4];

        // Prepare medians
        // TODO: allow restriction to a sample
        for (sample_idx, pix) in pixel_data.chunks(channels).enumerate() {
            for (i, p) in pix.iter().enumerate() {
                self.data.values[i * samples + sample_idx] = *p;
            }
        }

        // Calculate medians and inverse inter-quartile range
        for i in 0..channels {
            let slice = &mut self.data.values[(i * samples)..(i * samples + samples)];
            slice.sort_unstable();
            if threshold.absolute() {
                median[i] = Self::median(slice);
            } else {
                let (q1, med, q3) = Self::quartiles(slice);
                median[i] = med;
                iqr_inv[i] = q3 - q1;
                if iqr_inv[i] == 0.0 {
                    iqr_inv[i] = 1.0;
                }
                iqr_inv[i] = 1.0 / iqr_inv[i];
            }
        }

        let mut num_outliers = 0;
        let mut max_dist_sq = 0.0;
        let mut max_index = 0;

        for (sample_idx, pix) in pixel_data.chunks(channels).enumerate() {
            let mut dist_sq = 0.0;
            for (i, p) in pix.iter().enumerate() {
                let diff = median[i] - *p as f32;
                dist_sq += if diff == 0.0 {
                    0.0
                } else {
                    if threshold.absolute() {
                        (diff * diff)
                    } else {
                        (iqr_inv[i] * diff).powi(2)
                    }
                };
            }
            //println!("{:?}, {:?}", dist_sq.sqrt(), threshold_sq.sqrt());
            if dist_sq >= threshold_sq {
                self.data.outlier_indices[num_outliers] = (sample_idx, dist_sq);
                num_outliers += 1;
                if dist_sq > max_dist_sq {
                    max_dist_sq = dist_sq;
                    max_index = sample_idx;
                }
            }
        }

        let has_outliers = num_outliers > 0;
        let mut has_warning = false;

        // Fill pixel with background
        match self.background {
            BackgroundMode::Average => {
                let mut mean = [0.0; 4];
                for pix in pixel_data.chunks(channels) {
                    for (i, p) in pix.iter().enumerate() {
                        mean[i] += *p as f32;
                    }
                }
                for m in mean.iter_mut() {
                    *m /= samples as f32;
                }
                if has_outliers {
                    if num_outliers == 1 {
                        let offset = self.data.outlier_indices[0].0 * channels;
                        let sample = &pixel_data[offset..(offset + channels)];
                        for ch in 0..channels {
                            pixel[ch] = (mean[ch] * (samples as f32 / (samples - 1) as f32)
                                - sample[ch] as f32 / samples as f32)
                                .round() as u8;
                        }
                    } else {
                        let mut outlier_sum = [0.0; 4];
                        for (sample_idx, _dist_sq) in
                            self.data.outlier_indices.iter().take(num_outliers)
                        {
                            let offset = sample_idx * channels;
                            for ch in 0..channels {
                                outlier_sum[ch] += pixel_data[offset + ch] as f32;
                            }
                        }
                        // TODO: check the equation again!
                        let num_non_outliers = samples - num_outliers;
                        for ch in 0..channels {
                            pixel[ch] = (mean[ch] * (samples as f32 / num_non_outliers as f32)
                                - outlier_sum[ch] / samples as f32)
                                .round() as u8;
                        }
                    }
                } else {
                    for ch in 0..channels {
                        pixel[ch] = mean[ch].round() as u8;
                    }
                }
            }
            BackgroundMode::Median => {
                // In case of median, we don't remove the outliers!
                for ch in 0..channels {
                    pixel[ch] = median[ch].round() as u8;
                }
            }
            BackgroundMode::First | BackgroundMode::Random => {
                let (sample_idx, warning) = match self.background {
                    BackgroundMode::First => {
                        if !has_outliers {
                            (0, false)
                        } else {
                            self.first_excluded(samples, num_outliers).unwrap()
                        }
                    }
                    BackgroundMode::Random => {
                        if !has_outliers {
                            (self.data.rng.gen_range(0, samples), false)
                        } else {
                            self.sample_excluded(samples, num_outliers).unwrap()
                            /*
                            match self.sample_excluded(samples, num_outliers) {
                                Ok(value) => value,
                                Err(err) => {
                                    println!(
                                        "{:?}",
                                        &self.data.outlier_indices[..num_outliers]
                                            .iter()
                                            .map(|v| v.1.sqrt())
                                            .collect::<Vec<_>>()
                                    );
                                    for pix in pixel_data.chunks(channels) {
                                        println!("{:?}", pix);
                                    }
                                    println!("Median: {:?}", median);
                                    panic!("Problem at pixel {:?}: {:?}", coord, err)
                                }
                            }*/
                        }
                    }
                    _ => (0, false),
                };
                let sample =
                    &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];

                for ch in 0..channels {
                    pixel[ch] = sample[ch];
                }

                if warning {
                    has_warning = true;
                }
            }
        }

        if has_outliers {
            // Get outlier
            if num_outliers == 1 {
                // Only one outlier
                let (sample_idx, dist_sq) = self.data.outlier_indices[0];
                let sample =
                    &pixel_data[(sample_idx * channels)..(sample_idx * channels + channels)];

                let blend = threshold.blend_value(dist_sq.sqrt());
                color::blend_into_u8(&mut pixel, &sample, blend);
                ((blend * 255.0).round() as u8, has_warning)
            } else {
                // More outliers
                if self.outlier == OutlierSelectionMode::AllForward
                    || self.outlier == OutlierSelectionMode::AllBackward
                {
                    let mut pix_new = [0.0; 4];
                    let mut blend_inv = 1.0;
                    for ch in 0..channels {
                        pix_new[ch] = pixel[ch] as f32;
                    }
                    if self.outlier == OutlierSelectionMode::AllForward {
                        for (sample_idx, dist_sq) in
                            self.data.outlier_indices.iter().take(num_outliers)
                        {
                            let offset = sample_idx * channels;
                            let sample = &pixel_data[offset..(offset + channels)];
                            // Blend outlier into background
                            let blend = threshold.blend_value(dist_sq.sqrt());
                            color::blend_into_f32_u8(&mut pix_new, &sample, blend);
                            blend_inv *= 1.0 - blend;
                        }
                    } else {
                        for (sample_idx, dist_sq) in
                            self.data.outlier_indices.iter().take(num_outliers).rev()
                        {
                            let offset = sample_idx * channels;
                            let sample = &pixel_data[offset..(offset + channels)];
                            // Blend outlier into background
                            let blend = threshold.blend_value(dist_sq.sqrt());
                            color::blend_into_f32_u8(&mut pix_new, &sample, blend);
                            blend_inv *= 1.0 - blend;
                        }
                    }
                    for ch in 0..channels {
                        pixel[ch] = pix_new[ch].round() as u8;
                    }
                    (((1.0 - blend_inv) * 255.0).round() as u8, has_warning)
                } else {
                    let mut temp_sample = [0; 4];
                    let (sample, dist) = if self.outlier == OutlierSelectionMode::Average {
                        if num_outliers == 1 {
                            let (sample_idx, dist_sq) = self.data.outlier_indices[0];
                            let sample = &pixel_data
                                [(sample_idx * channels)..(sample_idx * channels + channels)];
                            (sample, dist_sq.sqrt())
                        } else {
                            let mut mean = [0.0; 4];
                            for pix in pixel_data.chunks(channels) {
                                for (i, p) in pix.iter().enumerate() {
                                    mean[i] += *p as f32;
                                }
                            }
                            for m in mean.iter_mut() {
                                *m /= samples as f32;
                            }

                            for ch in 0..channels {
                                mean[ch] = 0.0;
                            }
                            let mut mean_dist = 0.0;
                            for (sample_idx, dist_sq) in
                                self.data.outlier_indices.iter().take(num_outliers)
                            {
                                let offset = sample_idx * channels;
                                for ch in 0..channels {
                                    mean[ch] += pixel_data[offset + ch] as f32;
                                }
                                mean_dist += dist_sq.sqrt();
                            }
                            for ch in 0..channels {
                                temp_sample[ch] = (mean[ch] / num_outliers as f32).round() as u8;
                            }
                            (&temp_sample[..], mean_dist / num_outliers as f32)
                        }
                    } else {
                        let (sample_idx, dist_sq) = match self.outlier {
                            OutlierSelectionMode::First => self.data.outlier_indices[0],
                            OutlierSelectionMode::Last => {
                                self.data.outlier_indices[num_outliers - 1]
                            }
                            OutlierSelectionMode::Extreme => (max_index, max_dist_sq),
                            OutlierSelectionMode::Average
                            | OutlierSelectionMode::AllForward
                            | OutlierSelectionMode::AllBackward => (0, 0.0),
                        };
                        let sample = &pixel_data
                            [(sample_idx * channels)..(sample_idx * channels + channels)];
                        (sample, dist_sq.sqrt())
                    };
                    // Blend outlier into background
                    let blend = threshold.blend_value(dist);
                    color::blend_into_u8(&mut pixel, &sample, blend);
                    ((blend * 255.0).round() as u8, has_warning)
                }
            }
        } else {
            (0, has_warning)
        }
    }

    /// Returns the first index in 0..samples that does not appear in the outliers
    fn first_excluded(
        &self,
        samples: usize,
        num_outliers: usize,
    ) -> Result<(usize, bool), PixelSelectionError> {
        if num_outliers == samples {
            /*return Err(PixelSelectionError(
                "Unable to select random background pixel. All pixels seem to be outliers."
                    .to_string(),
            ));*/
            return Ok((0, true));
        }
        let excluded = &self.data.outlier_indices[..num_outliers];
        let len = excluded.len();
        let mut excl_index = 0;
        for i in 0..samples {
            if excl_index < len && i == excluded[excl_index].0 {
                excl_index += 1;
            } else {
                return Ok((i, false));
            }
        }
        Err(PixelSelectionError(
            "Unable to select first background pixel. All pixels seem to be outliers.".to_string(),
        ))
    }
    /// Returns a random index in 0..samples that does not appear in the outliers
    fn sample_excluded(
        &mut self,
        samples: usize,
        num_outliers: usize,
    ) -> Result<(usize, bool), PixelSelectionError> {
        if num_outliers == samples {
            /*return Err(PixelSelectionError(
                "Unable to select random background pixel. All pixels seem to be outliers."
                    .to_string(),
            ));*/
            return Ok((self.data.rng.gen_range(0_usize, samples), true));
        }
        let excluded = &self.data.outlier_indices[..num_outliers];
        for (i, idx) in self.data.non_outlier_indices.iter_mut().enumerate() {
            *idx = i;
        }
        let mut candidates = samples;
        for idx in excluded {
            self.data.non_outlier_indices.swap(idx.0, candidates - 1);
            candidates -= 1;
        }
        let idx = self.data.rng.gen_range(0_usize, candidates);
        Ok((self.data.non_outlier_indices[idx], false))
    }

    /// Calculates quartiles from a sample.
    /// Return (Q1, Median, Q3)
    fn quartiles(data: &[u8]) -> (f32, f32, f32) {
        (
            Self::quantile(data, 0.25),
            Self::median(data),
            Self::quantile(data, 0.75),
        )
    }

    /// Calculates a quantile a sample.
    fn quantile(data: &[u8], q: f32) -> f32 {
        let pos = (data.len() + 1) as f32 * q;
        let p1 = pos as usize - 1;
        let frac = pos.fract();
        if frac < 0.001 {
            data[p1] as f32
        } else if frac > 0.999 {
            data[p1 + 1] as f32
        } else {
            (1.0 - frac) * data[p1] as f32 + frac * data[p1 + 1] as f32
        }
    }

    /// Calculates the median of a sample.
    fn median(data: &[u8]) -> f32 {
        let len = data.len();

        if (len + 1) % 2 == 0 {
            data[(len + 1) / 2 - 1] as f32
        } else {
            let idx = (len + 1) / 2;
            0.5 * (data[idx - 1] as f32 + data[idx] as f32)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::chrono::ChronoProcessor;

    #[test]
    fn quartiles_test() {
        let values = [0, 1, 2, 3, 4, 5, 6];
        println!("{:?}", ChronoProcessor::quartiles(&values));

        assert_eq!(ChronoProcessor::quartiles(&values), (1.0, 3.0, 5.0))
    }
}
