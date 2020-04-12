use crate::color;
use crate::options::Fade;
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct SimpleProcessor {
    weights: [f32; 4],
    fade: Fade,
    darker: bool,
}

impl SimpleProcessor {
    pub fn new(weights: [f32; 4], fade: Fade, darker: bool) -> Self {
        SimpleProcessor {
            weights,
            fade,
            darker,
        }
    }

    pub fn process(
        self,
        files: &[PathBuf],
        image_indices: Option<&[usize]>,
    ) -> image::ImageResult<(Vec<u8>, SampleLayout)> {
        let samples = match image_indices {
            Some(indices) => indices.len(),
            None => files.len(),
        };
        let mut channels = None;
        let mut buffer = None;
        let mut extreme_value = None;

        let mut layout: Option<SampleLayout> = None;

        let mut fun = |sample_idx: usize, path: &PathBuf| -> image::ImageResult<()> {
            let image = image::open(path)?;
            let buff = image
                .as_flat_samples_u8()
                .expect("Unexpected format. Not an 8 bit image.");

            let frame_offset = match image_indices {
                Some(indices) => indices[0] as i32,
                None => 0,
            };

            // Prepare data
            match layout {
                Some(lay) => {
                    if buff.layout != lay {
                        // TODO better error handling
                        panic!("Image layout does not fit!".to_string());
                    }
                }
                None => {
                    layout = Some(buff.layout);
                    channels = Some(buff.layout.width_stride);
                    buffer = Some(vec![
                        0;
                        buff.layout.height as usize * buff.layout.height_stride
                    ]);
                    extreme_value = Some(vec![
                        if self.darker {
                            std::f32::MAX
                        } else {
                            std::f32::MIN
                        };
                        buff.layout.height as usize
                            * buff.layout.width as usize
                    ]);
                }
            };
            let channels = channels.unwrap();
            let extremes = extreme_value.as_mut().unwrap();

            /*for (idx, (out_pix, in_pix)) in buffer
                .as_mut()
                .unwrap()
                .par_chunks_mut(channels)
                .zip(buff.samples.par_chunks(channels))
                .enumerate()
            {*/
            buffer
                .as_mut()
                .unwrap()
                .par_chunks_mut(channels)
                .zip(buff.samples.par_chunks(channels))
                .zip(extremes.par_iter_mut())
                .for_each(|((out_pix, in_pix), extreme)| {
                    let mut value = 0.0;
                    for ch in 0..channels {
                        value += in_pix[ch] as f32 * self.weights[ch];
                    }
                    let mut is_extreme = false;
                    if self.darker {
                        //if value < extremes[idx] {
                        if value < *extreme {
                            is_extreme = true;
                        }
                    } else {
                        //if value > extremes[idx] {
                        if value > *extreme {
                            is_extreme = true;
                        }
                    }
                    if is_extreme {
                        //extremes[idx] = value;
                        *extreme = value;
                        let fade = self.fade(sample_idx as i32, samples as i32, frame_offset);
                        if fade > 0.0 {
                            if fade >= 1.0 {
                                for ch in 0..channels {
                                    out_pix[ch] = in_pix[ch];
                                }
                            } else {
                                color::blend_into_u8(out_pix, &in_pix, fade);
                            }
                        }
                    }
                });

            Ok(())
        };

        match image_indices {
            Some(indices) => {
                let bar = ProgressBar::new(indices.len() as u64);
                for (i, index) in indices.iter().enumerate() {
                    bar.inc(1);
                    fun(i, &files[*index])?;
                }
                bar.finish_and_clear();
            }
            None => {
                let bar = ProgressBar::new(files.len() as u64);
                for (i, file) in files.iter().enumerate() {
                    bar.inc(1);
                    fun(i, file)?;
                }
                bar.finish_and_clear();
            }
        }

        Ok((buffer.unwrap(), layout.unwrap()))
    }

    fn fade(&self, frame: i32, total: i32, offset: i32) -> f32 {
        if self.fade.absolute() {
            self.fade.get(offset + frame)
        } else {
            self.fade.get(total - frame - 1)
        }
    }
}
