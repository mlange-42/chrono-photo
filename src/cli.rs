//! Command-line interface for chrono-photo.
use crate::flist::FrameRange;
use crate::options::{BackgroundMode, Fade, OutlierSelectionMode, SelectionMode, Threshold};
use crate::shake::{ShakeAnchor, ShakeParams, ShakeReduction};
use crate::slicer::SliceLength;
use crate::streams::Compression;
use core::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Command-line tool for combining images into a single chrono-photograph or chrono-video.
///
/// Use `chrono-photo -h`     for help, or
///     `chrono-photo --help` for more detailed help.
///
/// For more documentation and explanation of the algorithm, see the GitHub repository:
///      https://mlange-42.github.io/chrono-photo/
#[derive(StructOpt)]
#[structopt(verbatim_doc_comment)]
//#[structopt(name = "chrono-photo command line application")]
pub struct Cli {
    /// File search pattern. ** MUST be quoted on Unix systems! **
    #[structopt(short, long)]
    pattern: String,

    // /// Sets to 16 bit per color channel mode. Use for 16 bit TIFF files.
    // #[structopt(long, name = "16-bit")]
    // is_16bit: bool,
    /// Frames to be used from those matching pattern: `start/end/step`. Optional.
    /// For default values, use `.`, e.g. `././2`.
    #[structopt(short, long, value_name = "frames")]
    frames: Option<FrameRange>,

    /// Video input frames. Frames to be used per video frame: `start/end/step`. Optional.
    #[structopt(long, name = "video-in", value_name = "frames")]
    video_in: Option<FrameRange>,

    /// Video output frames. Range and step width of video output frames: `start/end/step`. Optional.
    #[structopt(long, name = "video-out", value_name = "frames")]
    video_out: Option<FrameRange>,

    /// Path to output file
    #[structopt(short, long, value_name = "path")]
    output: String,

    /// Temp directory. Used with `--mode outlier` only. Optional, default system temp directory.
    #[structopt(long, name = "temp-dir", value_name = "path")]
    temp_dir: Option<String>,

    /// Path of output image showing which pixels are outliers (blend value).
    /// Used with `--mode outlier` only.
    #[structopt(long, name = "output-blend", value_name = "path")]
    output_blend: Option<String>,

    /// Pixel selection mode (lighter|darker|outlier). Optional, default 'outlier'.
    #[structopt(short, long)]
    mode: Option<SelectionMode>,

    /// Outlier threshold mode (abs|rel)/<lower>[/<upper>]. Optional, default 'abs/0.05/0.2'.
    /// Used with `--mode outlier` only.
    #[structopt(short, long, value_name = "thresh")]
    threshold: Option<Threshold>,

    /// Background pixel selection mode (first|random|average|median). Optional, default 'random'.
    /// Used with `--mode outlier` only.
    #[structopt(short, long, value_name = "bg")]
    background: Option<BackgroundMode>,

    /// Outlier selection mode in case more than one outlier is found
    /// (first|last|extreme|average|forward|backward). Optional, default 'extreme'.
    /// Used with `--mode outlier` only.
    #[structopt(short = "l", long, value_name = "mode")]
    outlier: Option<OutlierSelectionMode>,

    /// Compression mode and level (0 to 9) for time slices (gzip|zlib|deflate)[/<level>].
    /// Used with `--mode outlier` only.
    /// Optional, default 'gzip/6'.
    #[structopt(short, long, value_name = "comp/lev")]
    compression: Option<Compression>,

    /// Output image quality for JPG files, in percent. Optional, default '95'.
    #[structopt(short, long)]
    quality: Option<u8>,

    /// Controls slicing to temp files (rows|pixels|count)/<number>.
    /// Used with `--mode outlier` only.
    /// Optional, default 'rows/4'.
    #[structopt(short, long)]
    slice: Option<SliceLength>,

    /// Restricts calculation of median and inter-quartile range to a sub-sample of input images.
    /// Use for large amounts of images to speed up calculations. Optional.
    /// Used with `--mode outlier` only.
    #[structopt(long)]
    sample: Option<usize>,

    /// Color channel weights (4 values: RGBA) for distance calculation. Optional, default '1 1 1 1'.
    #[structopt(long, number_of_values = 4, value_name = "w")]
    weights: Option<Vec<f32>>,

    /// Frame fading. Optional, default None. Format: (clamp|repeat)/(abs|rel)/(f1,v1)/(f2,v2)[/(f,v)...]
    #[structopt(long)]
    fade: Option<Fade>,

    /// Number of threads. Optional, default equal to number of processors.
    #[structopt(long, value_name = "num")]
    threads: Option<usize>,

    /// Number of threads for parallel video frame output. Optional, default equal to number of processors.
    /// Limiting this may be required if memory usage is too high.
    #[structopt(long, name = "video-threads", value_name = "num")]
    video_threads: Option<usize>,

    /// Camera shake reduction parameters. Optional, default none.
    /// Format: `anchor-radius/shake-radius`
    #[structopt(long, value_name = "r1/r2")]
    shake: Option<ShakeParams>,

    /// Camera shake reduction anchors. Optional, default none. Format: `x1/y1 [x2/y2 [...]]`
    #[structopt(long, name = "shake-anchors", value_name = "x/y")]
    shake_anchors: Option<Vec<ShakeAnchor>>,

    /// Prints debug information (i.e. parsed cmd parameters) before processing.
    #[structopt(long, short)]
    debug: bool,

    /// Keeps the terminal open after processing and waits for user key press.
    #[structopt(long, short)]
    wait: bool,
}

impl Cli {
    /// Parses this Cli into a [CliParsed](struct.CliParsed.html).
    pub fn parse(self) -> Result<CliParsed, ParseCliError> {
        let mut warings = Vec::new();
        if self.mode.is_some() && self.mode.as_ref().unwrap() != &SelectionMode::Outlier {
            if self.output_blend.is_some() {
                warings.push("--output-blend".to_string());
            }
            if self.threshold.is_some() {
                warings.push("--threshold".to_string());
            }
            if self.outlier.is_some() {
                warings.push("--outlier".to_string());
            }
            if self.background.is_some() {
                warings.push("--background".to_string());
            }
            if self.temp_dir.is_some() {
                warings.push("--temp-dir".to_string());
            }
            if self.sample.is_some() {
                warings.push("--sample".to_string());
            }
            if self.slice.is_some() {
                warings.push("--slice".to_string());
            }
            if self.compression.is_some() {
                warings.push("--compression".to_string());
            }
        }
        if self.shake.is_some() != self.shake_anchors.is_some() {
            return Err(ParseCliError(
                "Provide both options or none: `--shake` and `--shake-anchors`".to_string(),
            ));
        }

        let mut weights = [1.0; 4];
        if let Some(w) = &self.weights {
            for (i, v) in w.iter().enumerate() {
                weights[i] = *v;
            }
        }

        let shake_params = self.shake;
        let shake_anchors = self.shake_anchors;
        let out = CliParsed {
            pattern: self.pattern,
            // is_16bit: self.is_16bit,
            temp_dir: self.temp_dir.map(|d| PathBuf::from(d)),
            output: PathBuf::from(&self.output),
            output_blend: match self.output_blend {
                Some(out) => Some(PathBuf::from(out)),
                None => None,
            },
            mode: self.mode.unwrap_or(SelectionMode::Outlier),
            threshold: self.threshold.unwrap_or(Threshold::abs(0.05, 0.2)),
            background: self.background.unwrap_or(BackgroundMode::Random),
            outlier: self.outlier.unwrap_or(OutlierSelectionMode::Extreme),
            compression: self.compression.unwrap_or(Compression::GZip(6)),
            quality: match self.quality {
                Some(q) => {
                    if q <= 100 && q > 0 {
                        q
                    } else {
                        return Err(ParseCliError(format!(
                            "Expected 0 < qualtiy <= 100. Got value {}",
                            q
                        )));
                    }
                }
                None => 95,
            },
            frames: self.frames,
            video_in: self.video_in,
            video_out: self.video_out,
            slice: self.slice.unwrap_or(SliceLength::Rows(4)),
            sample: self.sample,
            weights,
            fade: self.fade.unwrap_or(Fade::none()),
            threads: self.threads,
            video_threads: self.video_threads,
            shake_reduction: shake_params.and_then(|shake| {
                shake_anchors.and_then(|anchors| {
                    Some(ShakeReduction::new(
                        anchors.iter().map(|a| a.anchor()).collect(),
                        shake.anchor_radius(),
                        shake.search_radius(),
                    ))
                })
            }),
            debug: self.debug,
            wait: self.wait,
        };

        if !warings.is_empty() {
            println!("WARNING! The following options are not used, as they are required only for `--mode outlier`:", );
            for w in warings {
                println!("{}", w);
            }
            println!();
        }

        out.validate()
    }
}

impl FromStr for Cli {
    type Err = ParseCliError;

    /// Parses a string into a Cli.
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let quote_parts: Vec<_> = str.split('"').collect();
        let mut args: Vec<String> = vec![];
        for (i, part) in quote_parts.iter().enumerate() {
            let part = part.trim();
            if i % 2 == 0 {
                args.extend(
                    part.split(' ')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                );
            } else {
                args.push(part.to_string());
            }
        }
        Ok(Cli::from_iter(args.iter()))
    }
}

/// Parsed command line arguments.
#[allow(dead_code)]
#[derive(Debug)]
pub struct CliParsed {
    /// File search pattern
    pub pattern: String,
    // /// Sets to 16 bit per color channel mode. Use for 16 bit TIFF files.
    //pub is_16bit: bool,
    /// Frames to be used from those matching pattern: `start/end/step`. Optional.
    /// For default values, use `.`, e.g. `././step`.
    pub frames: Option<FrameRange>,
    /// Video input frames. Frames to be used per video frame: `start/end/step`. Optional.
    pub video_in: Option<FrameRange>,
    /// Video output frames. Range and step width of video output frames: `start/end/step`. Optional.
    pub video_out: Option<FrameRange>,
    /// Temp directory. Uses system temp directory if `None`.
    pub temp_dir: Option<PathBuf>,
    /// Path of the final output image.
    pub output: PathBuf,
    /// Path of output image showing which pixels are outliers (blend value).
    pub output_blend: Option<PathBuf>,
    /// Pixel selection mode.
    pub mode: SelectionMode,
    /// Outlier threshold mode.
    pub threshold: Threshold,
    /// Outlier selection mode in case more than one outlier is found.
    pub outlier: OutlierSelectionMode,
    /// Background pixel selection mode.
    pub background: BackgroundMode,
    /// Compression mode for time slices.
    pub compression: Compression,
    /// Output image quality for JPG files, in percent.
    pub quality: u8,
    /// Controls slicing to temp files (rows|pixels|count)/<number>. Optional, default 'rows/1'
    pub slice: SliceLength,
    /// Restricts calculation of median and inter-quartile range to a sub-sample of input images. Use for large amounts of images to speed up calculations. Optional.
    pub sample: Option<usize>,
    /// Color channel weights for distance calculation
    pub weights: [f32; 4],
    /// Frame fading. Optional, default None.
    pub fade: Fade,
    /// Number of threads. Optional, default equal to number of processors.
    pub threads: Option<usize>,
    /// Number of threads for parallel video frame output. Optional, default equal to number of processors.
    pub video_threads: Option<usize>,
    /// Shake reduction
    pub shake_reduction: Option<ShakeReduction>,
    /// Print debug information (i.e. parsed cmd parameters).
    pub debug: bool,

    /// Keep the terminal open after processing and wait for user key press.
    pub wait: bool,
}

impl CliParsed {
    /// Check for validity
    pub fn validate(self) -> Result<Self, ParseCliError> {
        Ok(self)
    }
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCliError(String);

impl fmt::Display for ParseCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;

    #[test]
    fn args_from_string() {
        let str = "chrono-photo --pattern \"test_data/generated/*.jpg\" --output test_data/temp --weights 0 1 1 0";
        let cli: Cli = str.parse().unwrap();
        let parsed = cli.parse().unwrap();

        //println!("{:#?}", parsed);
    }
}
