//! Command-line interface for chrono-photo.
use crate::flist::FrameRange;
use crate::options::{BackgroundMode, Fade, OutlierSelectionMode, SelectionMode, Threshold};
use crate::slicer::SliceLength;
use crate::streams::Compression;
use core::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

/// Command-line tool for combining images into a single chrono-photograph or chrono-video.
///
/// Use `chrono-photo -h`     for help, or
///     `chrono-photo --help` even more comprehensive help.
#[derive(StructOpt)]
#[structopt(verbatim_doc_comment)]
//#[structopt(name = "chrono-photo command line application")]
pub struct Cli {
    /// File search pattern
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

    /// Temp directory. Optional, default system temp directory.
    #[structopt(short = "d", long, name = "temp-dir", value_name = "path")]
    temp_dir: Option<String>,

    /// Path of output image showing which pixels are outliers (blend value).
    #[structopt(long, name = "output-blend", value_name = "path")]
    output_blend: Option<String>,

    /// Pixel selection mode (lighter|darker|outlier). Optional, default 'outlier'.
    #[structopt(short, long)]
    mode: Option<SelectionMode>,

    /// Outlier threshold mode (abs|rel)/<lower>[/<upper>]. Optional, default 'abs/0.05/0.2'.
    #[structopt(short, long, value_name = "thresh")]
    threshold: Option<Threshold>,

    /// Background pixel selection mode (first|random|average|median). Optional, default 'random'.
    #[structopt(short, long, value_name = "bg")]
    background: Option<BackgroundMode>,

    /// Outlier selection mode in case more than one outlier is found (first|last|extreme|average|forward|backward). Optional, default 'extreme'.
    #[structopt(short = "l", long, value_name = "mode")]
    outlier: Option<OutlierSelectionMode>,

    /// Compression mode and level (0 to 9) for time slices (gzip|zlib|deflate)[/<level>]. Optional, default 'gzip/6'.
    #[structopt(short, long, value_name = "comp/lev")]
    compression: Option<Compression>,

    /// Output image quality for JPG files, in percent. Optional, default '95'.
    #[structopt(short, long)]
    quality: Option<u8>,

    /// Controls slicing to temp files (rows|pixels|count)/<number>. Optional, default 'rows/4'.
    #[structopt(short, long)]
    slice: Option<SliceLength>,

    /// Restricts calculation of median and inter-quartile range to a sub-sample of input images. Use for large amounts of images to speed up calculations. Optional.
    #[structopt(long)]
    sample: Option<usize>,

    /// Color channel weights (4 values: RGBA) for distance calculation. Optional, default '1 1 1 1'.
    #[structopt(long, short, number_of_values = 4, value_name = "w")]
    weights: Option<Vec<f32>>,

    /// Frame fading. Optional, default None. Format: (clamp|repeat)/(abs|rel)/(f1,v1)/(f2,v2)[/(f,v)...]
    #[structopt(long)]
    fade: Option<Fade>,

    /// Print debug information (i.e. parsed cmd parameters).
    #[structopt(long)]
    debug: bool,
}

impl Cli {
    /// Parses this Cli into a [CliParsed](struct.CliParsed.html).
    pub fn parse(self) -> Result<CliParsed, ParseCliError> {
        let mut weights = [1.0; 4];
        if let Some(w) = &self.weights {
            for (i, v) in w.iter().enumerate() {
                weights[i] = *v;
            }
        }
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
            debug: self.debug,
        };
        out.validate()
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
    /// Print debug information (i.e. parsed cmd parameters).
    pub debug: bool,
}

impl CliParsed {
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
