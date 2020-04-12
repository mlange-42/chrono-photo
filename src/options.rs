use crate::{ParseEnumError, ParseOptionError};
use std::str::FromStr;

/// Pixel selection mode.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    /// Selects by outlier analysis (multi-dimensional z-score).
    /// Parameter `threshold` determines the minimum distance from the median, in fractions of the total color range (i.e. [0, 1]), to classify a pixel as outlier.
    Outlier,
    /// Selects the lightest/brightest pixel (sum of red, green and blue).
    Lighter,
    /// Selects the darkest pixel (sum of red, green and blue).
    Darker,
}
impl FromStr for SelectionMode {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
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

#[derive(Debug, PartialEq, Clone)]
pub enum FadeMode {
    Repeat,
    Clamp,
    //RepeatMirror, TODO
}

impl FromStr for FadeMode {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "repeat" => Ok(FadeMode::Repeat),
            "clamp" => Ok(FadeMode::Clamp),
            _ => Err(ParseEnumError(format!(
                "Not a fade mode: {}. Must be one of (repeat|clamp)",
                str
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fade {
    is_none: bool,
    mode: FadeMode,
    absolute: bool,
    offset: i32,
    values: Vec<f32>,
}
impl Fade {
    /// Expects vector of (frame, value) pairs, ordered by frame
    pub fn new(mode: FadeMode, absolute: bool, frames: Vec<(i32, f32)>) -> Self {
        let offset = frames[0].0;
        let len = (frames
            .last()
            .expect("Fade requires at least two frames specified.")
            .0
            - offset) as usize;
        let mut values = vec![1.0; len + 1];
        let mut idx = 0;
        for i in 0..(len + 1) {
            let (f1, v1) = frames[idx];
            let (f2, v2) = frames[idx + 1];
            let frame = i as i32 + offset;
            values[i] = v1 + (v2 - v1) * (frame - f1) as f32 / (f2 - f1) as f32;
            if frame == f2 {
                idx += 1;
            }
        }
        Fade {
            is_none: false,
            mode,
            absolute,
            offset,
            values,
        }
    }

    pub fn none() -> Self {
        Fade {
            is_none: true,
            mode: FadeMode::Clamp,
            absolute: true,
            offset: 0,
            values: vec![],
        }
    }

    pub fn absolute(&self) -> bool {
        self.absolute
    }

    pub fn get(&self, frame: i32) -> f32 {
        if self.is_none {
            return 1.0;
        }
        let mut i = frame - self.offset;
        let len = self.values.len();
        if i >= 0 && i < len as i32 {
            self.values[i as usize]
        } else {
            match self.mode {
                FadeMode::Clamp => {
                    if i < 0 {
                        self.values[0]
                    } else {
                        self.values[len - 1]
                    }
                }
                FadeMode::Repeat => {
                    while i < 0 {
                        i += len as i32;
                    }
                    i = i % len as i32;
                    self.values[i as usize]
                }
            }
        }
    }
}
impl FromStr for Fade {
    type Err = ParseOptionError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        let mode_str = parts
            .get(0)
            .expect(&format!("Unexpected format in {}", str));
        let mode = mode_str.parse().expect(&format!(
            "Unexpected fade mode in {}, must be one of (clamp|repeat)",
            str
        ));

        let abs_str = parts
            .get(1)
            .expect(&format!("Unexpected format in {}", str));
        let absolute = match *abs_str {
            "absolute" | "abs" => true,
            "relative" | "rel" => false,
            _ => return Err(ParseOptionError(format!("Not a frame fade spec: {}", str))),
        };
        let mut frames = vec![];
        for p in parts.iter().skip(2) {
            let parts: Vec<_> = p.split(',').collect();
            if parts.len() != 2 {
                return Err(ParseOptionError(format!(
                    "Expected (int,float) per frame for fade. Got: {}",
                    str
                )));
            }
            frames.push((
                parts[0].parse().expect(&format!(
                    "Expected (int,float) per frame for fade. Got: {}",
                    str
                )),
                parts[1].parse().expect(&format!(
                    "Expected (int,float) per frame for fade. Got: {}",
                    str
                )),
            ));
        }

        Ok(Fade::new(mode, absolute, frames))
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
        if absolute {
            Threshold {
                absolute,
                min: min * 255.0,
                max: max * 255.0,
                scale: 1.0 / ((max - min) * 255.0),
            }
        } else {
            Threshold {
                absolute,
                min,
                max,
                scale: 1.0 / (max - min),
            }
        }
    }
    pub fn abs(min: f32, max: f32) -> Self {
        Threshold::new(true, min, max)
    }
    pub fn rel(min: f32, max: f32) -> Self {
        Threshold::new(false, min, max)
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
    pub fn absolute(&self) -> bool {
        self.absolute
    }
    pub fn min(&self) -> f32 {
        self.min
    }
    pub fn max(&self) -> f32 {
        self.max
    }
}
impl FromStr for Threshold {
    type Err = ParseOptionError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        let opt_str = parts
            .get(0)
            .expect(&format!("Unexpected format in {}", str));
        let absolute = match *opt_str {
            "absolute" | "abs" => true,
            "relative" | "rel" => false,
            _ => return Err(ParseOptionError(format!(
                "Not a pixel outlier detection mode: {}. Must be one of (abs[olute]|rel[ative])/<min>[/<max>]",
                str
            ))),
        };

        let thresh_min_str = parts
            .get(1)
            .expect(&format!("Unexpected format in {}", str));
        let min = thresh_min_str.parse().expect(&format!(
            "Unable to parse lower threshold for outlier detection: {}",
            str
        ));
        let thresh_max_str = parts.get(2);
        let max = match thresh_max_str {
            Some(str) => str.parse().expect(&format!(
                "Unable to parse upper threshold for outlier detection: {}",
                str
            )),
            None => min,
        };

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
    /// Progressively blend all outliers into background, forward.
    AllForward,
    /// Progressively blend all outliers into background, backward.
    AllBackward,
}
impl FromStr for OutlierSelectionMode {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "first" => Ok(OutlierSelectionMode::First),
            "last" => Ok(OutlierSelectionMode::Last),
            "extreme" => Ok(OutlierSelectionMode::Extreme),
            "average" => Ok(OutlierSelectionMode::Average),
            "forward" => Ok(OutlierSelectionMode::AllForward),
            "backward" => Ok(OutlierSelectionMode::AllBackward),
            _ => Err(ParseEnumError(format!(
                "Not an outlier selection mode: {}. Must be one of (first|last|extreme|average|forward|backward)",
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
impl FromStr for BackgroundMode {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
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

#[cfg(test)]
mod test {
    use crate::options::Fade;

    #[test]
    fn fade_test() {
        let str = "clamp/abs/0,0/10,1";
        let _f: Fade = str.parse().unwrap();

        //println!("{:#?}", f);
    }
}
