//! Camera shake correction

use crate::ParseOptionError;
use image;
use image::FlatSamples;
use indicatif::ProgressBar;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ShakeReduction {
    anchors: Vec<(i32, i32)>,
    anchor_radius: u32,
    search_radius: u32,
}
impl ShakeReduction {
    pub fn new(anchors: Vec<(i32, i32)>, anchor_radius: u32, search_radius: u32) -> Self {
        ShakeReduction {
            anchors,
            anchor_radius,
            search_radius,
        }
    }
    pub fn anchors(&self) -> &[(i32, i32)] {
        &self.anchors[..]
    }
    pub fn anchor_radius(&self) -> u32 {
        self.anchor_radius
    }
    pub fn search_radius(&self) -> u32 {
        self.search_radius
    }
}
#[derive(Debug, Clone)]
pub struct ShakeParams {
    anchor_radius: u32,
    search_radius: u32,
}
impl ShakeParams {
    pub fn anchor_radius(&self) -> u32 {
        self.anchor_radius
    }
    pub fn search_radius(&self) -> u32 {
        self.search_radius
    }
}
impl FromStr for ShakeParams {
    type Err = ParseOptionError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        if parts.len() != 2 {
            return Err(ParseOptionError(format!(
                "Unexpected format in shake parameters, expected <rad>/<search-rad>: {}",
                str
            )));
        }
        let rad = parts
            .get(0)
            .unwrap()
            .parse()
            .expect(&format!("Unexpected format in shake parameter: {}", str));
        let search_rad = parts
            .get(1)
            .unwrap()
            .parse()
            .expect(&format!("Unexpected format in shake parameter: {}", str));

        Ok(ShakeParams {
            anchor_radius: rad,
            search_radius: search_rad,
        })
    }
}
#[derive(Debug, Clone)]
pub struct ShakeAnchor {
    anchor: (i32, i32),
}
impl ShakeAnchor {
    pub fn anchor(&self) -> (i32, i32) {
        self.anchor
    }
}
impl FromStr for ShakeAnchor {
    type Err = ParseOptionError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        if parts.len() != 2 {
            return Err(ParseOptionError(format!(
                "Unexpected format in shake anchor, expected x/y: {}",
                str
            )));
        }
        let x = parts.get(0).unwrap().parse().expect(&format!(
            "Unexpected format in shake anchor, expected x/y: {}",
            str
        ));
        let y = parts.get(1).unwrap().parse().expect(&format!(
            "Unexpected format in shake anchor, expected x/y: {}",
            str
        ));

        Ok(ShakeAnchor { anchor: (x, y) })
    }
}

pub struct ShakeAnalyzer {}

impl ShakeAnalyzer {
    pub fn analyze(
        &self,
        files: &[PathBuf],
        anchors: &[(i32, i32)],
        anchor_radius: u32,
        search_radius: u32,
        show_progress: bool,
    ) -> image::ImageResult<Vec<((i32, i32), i32)>> {
        let size = (2 * anchor_radius + 1) as i32;
        let search_size = (2 * search_radius + 1) as i32;
        let window_len = (size * size) as usize;
        let mut windows: Option<Vec<u8>> = None;
        let mut diffs = vec![0; (search_size * search_size) as usize];

        if show_progress {
            println!("Analyzing camera shake...");
        }
        let mut result = Vec::with_capacity(files.len());
        let bar = ProgressBar::new(files.len() as u64);
        for (i, file) in files.iter().enumerate() {
            if show_progress {
                bar.inc(1);
            }
            let image = image::open(file)?;
            match &windows {
                Some(_) => {}
                None => {
                    let ch = image
                        .as_flat_samples_u8()
                        .expect(&format!(
                            "Problem converting image {:?}: not 8 bits per channel",
                            file
                        ))
                        .layout
                        .width_stride;
                    let wins = vec![0; window_len * ch];
                    windows = Some(wins);
                    //channels = Some(ch);
                }
            };
            let mut wins = windows.as_mut().unwrap();
            if i == 0 {
                self.fill_windows(
                    &image.as_flat_samples_u8().expect(&format!(
                        "Problem converting image {:?}: not 8 bits per channel",
                        file
                    )),
                    anchors,
                    &mut wins,
                    anchor_radius,
                );
                result.push(((0, 0), 0));
            } else {
                self.calc_diffs(
                    &image.as_flat_samples_u8().expect(&format!(
                        "Problem converting image {:?}: not 8 bits per channel",
                        file
                    )),
                    anchors,
                    &wins[..],
                    &mut diffs[..],
                    anchor_radius,
                    search_radius,
                );
                let (min_idx, min_diff) =
                    diffs.iter().enumerate().min_by_key(|(_i, &d)| d).unwrap();
                let xmin = (min_idx as i32 % search_size) - search_radius as i32;
                let ymin = (min_idx as i32 / search_size) - search_radius as i32;

                result.push(((xmin, ymin), *min_diff));
            }
        }
        if show_progress {
            bar.finish_and_clear();
        }

        Ok(result)
    }

    fn fill_windows(
        &self,
        image: &FlatSamples<&[u8]>,
        anchors: &[(i32, i32)],
        windows: &mut [u8],
        anchor_radius: u32,
    ) {
        let size = (2 * anchor_radius + 1) as i32;
        let channels = image.layout.width_stride;
        let win_len = (size * size * channels as i32) as usize;
        for (i, (cx, cy)) in anchors.iter().enumerate() {
            let win = &mut windows[(i * win_len)..(i * win_len + win_len)];

            for dy in 0..size {
                let yy = *cy + dy - anchor_radius as i32;
                for dx in 0..size {
                    let xx = *cx + dx - anchor_radius as i32;
                    let idx = (dy * size + dx) * channels as i32;
                    let idx_image = image
                        .layout
                        .index(0, xx as u32, yy as u32)
                        .expect(&format!("Image coordinate out of range: {:?}", (xx, yy)));
                    for ch in 0..channels {
                        win[idx as usize + ch] = image.samples[idx_image + ch];
                    }
                }
            }
        }
    }

    fn calc_diffs(
        &self,
        image: &FlatSamples<&[u8]>,
        anchors: &[(i32, i32)],
        windows: &[u8],
        diff: &mut [i32],
        anchor_radius: u32,
        search_radius: u32,
    ) {
        let size = (2 * anchor_radius + 1) as i32;
        let search_size = (2 * search_radius + 1) as i32;
        let channels = image.layout.width_stride;
        let win_len = (size * size * channels as i32) as usize;
        for i in 0..diff.len() {
            diff[i] = 0;
        }
        for (i, (cx, cy)) in anchors.iter().enumerate() {
            let win = &windows[(i * win_len)..(i * win_len + win_len)];
            for ox in 0..search_size {
                for oy in 0..search_size {
                    let diff_idx = (ox * search_size + oy) as i32;
                    for dy in 0..size {
                        let yy = *cy + (oy - search_radius as i32) + dy - anchor_radius as i32;
                        for dx in 0..size {
                            let xx = *cx + (ox - search_radius as i32) + dx - anchor_radius as i32;
                            let idx = (dy * size + dx) * channels as i32;
                            let idx_image = image
                                .layout
                                .index(0, xx as u32, yy as u32)
                                .expect(&format!("Image coordinate out of range: {:?}", (xx, yy)));
                            for ch in 0..channels {
                                diff[diff_idx as usize] += (win[idx as usize + ch] as i32
                                    - image.samples[idx_image + ch] as i32)
                                    .pow(2);
                            }
                        }
                    }
                }
            }
        }
    }
}
