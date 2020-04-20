//! Camera shake correction

use crate::ParseOptionError;
use image;
use image::flat::SampleLayout;
use image::{DynamicImage, FlatSamples};
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;

/// Camera shake reduction parameters and anchors
#[derive(Debug, Clone)]
pub struct ShakeReduction {
    anchors: Vec<(i32, i32)>,
    anchor_radius: u32,
    search_radius: u32,
}
impl ShakeReduction {
    /// Creates a new ShakeReduction struct
    pub fn new(anchors: Vec<(i32, i32)>, anchor_radius: u32, search_radius: u32) -> Self {
        ShakeReduction {
            anchors,
            anchor_radius,
            search_radius,
        }
    }
    /// The shake reduction anchors
    pub fn anchors(&self) -> &[(i32, i32)] {
        &self.anchors[..]
    }
    /// The anchor radius
    pub fn anchor_radius(&self) -> u32 {
        self.anchor_radius
    }
    /// The search radius
    pub fn search_radius(&self) -> u32 {
        self.search_radius
    }
}

/// Camera shake reduction parameters
#[derive(Debug, Clone)]
pub struct ShakeParams {
    anchor_radius: u32,
    search_radius: u32,
}
impl ShakeParams {
    /// The anchor radius
    pub fn anchor_radius(&self) -> u32 {
        self.anchor_radius
    }
    /// The search radius
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

/// Camera shake reduction anchor
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

/// Image crop parameters
#[derive(Debug, Clone)]
pub struct Crop {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Crop {
    /// Creates new crop parameters
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Crop { x, y, w, h }
    }
    /// Creates a vector of crop parameters for each frame, from shake offsets and the original image's layout
    pub fn create(offset: &[(i32, i32)], layout: &SampleLayout) -> Option<Vec<Self>> {
        let mut xmin = 0;
        let mut ymin = 0;
        let mut xmax = 0;
        let mut ymax = 0;
        for (x, y) in offset {
            if *x < xmin {
                xmin = *x;
            }
            if *y < ymin {
                ymin = *y;
            }
            if *x > xmax {
                xmax = *x;
            }
            if *y > ymax {
                ymax = *y;
            }
        }
        if xmin == 0 && ymin == 0 && xmax == 0 && ymax == 0 {
            None
        } else {
            let w = layout.width as i32 + xmin - xmax;
            let h = layout.height as i32 + ymin - ymax;
            Some(
                offset
                    .iter()
                    .map(|(dx, dy)| {
                        //println!("{}-{}, {}-{}, {:?}", xmin, xmax, ymin, ymax, (dx, dy));
                        //println!("-> {:?}", (((-xmin) + *dx), ((-ymin) + *dy)));
                        Crop::new(
                            ((-xmin) + *dx) as u32,
                            ((-ymin) + *dy) as u32,
                            w as u32,
                            h as u32,
                        )
                    })
                    .collect(),
            )
        }
    }
    /// Crops an image with the parameters given by this
    pub fn crop(&self, image: &mut DynamicImage) -> DynamicImage {
        image.crop(self.x, self.y, self.w, self.h)
    }
}

/// Determines shake offsets
pub struct ShakeAnalyzer {}

impl ShakeAnalyzer {
    /// Analyze images to determine individual shake offsets
    /// # Returns
    /// A `Result` with offsets and the original image layout
    pub fn analyze(
        &self,
        files: &[PathBuf],
        anchors: &[(i32, i32)],
        anchor_radius: u32,
        search_radius: u32,
        threads: Option<usize>,
        show_progress: bool,
    ) -> image::ImageResult<(Vec<(i32, i32)>, SampleLayout)> {
        let size = (2 * anchor_radius + 1) as i32;
        let search_size = (2 * search_radius + 1) as i32;
        let window_len = (size * size) as usize;
        //let mut windows: Option<Vec<u8>> = None;
        //let mut layout: Option<SampleLayout> = None;

        if show_progress {
            println!("Analyzing camera shake...");
        }
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_draw_delta((files.len() / 200) as u64);

        let pool = match threads {
            Some(threads) => rayon::ThreadPoolBuilder::new().num_threads(threads),
            None => rayon::ThreadPoolBuilder::new(),
        }
        .build()
        .expect("Unable to build thread pool.");

        let first_image = image::open(&files[0])?;
        let (windows, layout) = {
            let lay = first_image
                .as_flat_samples_u8()
                .expect(&format!(
                    "Problem converting image {:?}: not 8 bits per channel",
                    &files[0]
                ))
                .layout;
            let ch = lay.width_stride;
            let mut wins = vec![0; anchors.len() * window_len * ch];

            self.fill_windows(
                &first_image.as_flat_samples_u8().expect(&format!(
                    "Problem converting image {:?}: not 8 bits per channel",
                    &files[0]
                )),
                anchors,
                &mut wins,
                anchor_radius,
            );
            (wins, lay)
        };

        let result: Vec<_> = pool.install(|| {
            files
                .par_iter()
                .skip(1)
                .map(|file| -> image::ImageResult<(i32, i32)> {
                    if show_progress {
                        bar.inc(1);
                    }
                    let image = image::open(file)?;
                    /*match &windows {
                        Some(_) => {}
                        None => {
                            let lay = image
                                .as_flat_samples_u8()
                                .expect(&format!(
                                    "Problem converting image {:?}: not 8 bits per channel",
                                    file
                                ))
                                .layout;
                            let ch = lay.width_stride;
                            let wins = vec![0; anchors.len() * window_len * ch];
                            windows = Some(wins);
                            layout = Some(lay);
                            //channels = Some(ch);
                        }
                    };*/
                    //let wins = windows.as_ref().unwrap();
                    /*if i == 0 {
                        self.fill_windows(
                            &image.as_flat_samples_u8().expect(&format!(
                                "Problem converting image {:?}: not 8 bits per channel",
                                file
                            )),
                            anchors,
                            &mut wins,
                            anchor_radius,
                        );
                        Ok((0, 0))
                    } else {*/

                    let mut diffs = vec![0; (search_size * search_size) as usize];
                    self.calc_diffs(
                        &image.as_flat_samples_u8().expect(&format!(
                            "Problem converting image {:?}: not 8 bits per channel",
                            file
                        )),
                        anchors,
                        &windows[..],
                        &mut diffs[..],
                        anchor_radius,
                        search_radius,
                    );
                    let (min_idx, _min_diff) =
                        diffs.iter().enumerate().min_by_key(|(_i, &d)| d).unwrap();
                    let xmin = (min_idx as i32 % search_size) - search_radius as i32;
                    let ymin = (min_idx as i32 / search_size) - search_radius as i32;

                    Ok((xmin, ymin))
                    //}
                })
                .collect()
        });
        if show_progress {
            bar.finish_and_clear();
        }
        let mut out = Vec::with_capacity(result.len());
        out.push((0, 0));
        for res in result {
            out.push(res?);
        }
        Ok((out, layout))
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
            for oy in 0..search_size {
                for ox in 0..search_size {
                    let diff_idx = (oy * search_size + ox) as i32;
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
