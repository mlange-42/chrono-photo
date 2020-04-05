use crate::{ParseEnumError, ParseOptionError};
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
