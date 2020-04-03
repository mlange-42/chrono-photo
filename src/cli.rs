//! Command-line interface for chrono-photo.
use crate::chrono::{BackgroundMode, OutlierSelectionMode, SelectionMode};
use crate::img_stream::Compression;
use crate::EnumFromString;
use core::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

/// Raw command line arguments.
#[derive(StructOpt)]
#[structopt(name = "chrono-photo command line application")]
pub struct Cli {
    /// File search pattern
    #[structopt(short, long)]
    pattern: String,
    /// Temp directory. Optional, default system temp directory.
    #[structopt(short, long, name = "temp-dir")]
    temp_dir: Option<String>,
    /// Path to output file
    #[structopt(short, long)]
    output: String,
    /// Pixel selection mode (lighter|darker|outlier-<threshold>). Optional, default 'outlier-3.0'.
    #[structopt(short, long)]
    mode: Option<String>,
    /// Background pixel selection mode (first|random|average). Optional, default 'first'.
    #[structopt(short, long)]
    background: Option<String>,
    /// Outlier selection mode in case more than one outlier is found (first|last|extreme|average). Optional, default 'extreme'.
    #[structopt(short = "l", long)]
    outlier: Option<String>,
    /// Compression mode for time slices (gzip|zlib|deflate). Optional, default 'gzip'.
    #[structopt(short, long)]
    compression: Option<String>,
    /// Output image quality for JPG files, in percent. Optional, default '95'.
    #[structopt(short, long)]
    quality: Option<u8>,
    /// Print debug information (i.e. parsed cmd parameters).
    #[structopt(short, long)]
    debug: bool,
}

impl Cli {
    /// Parses this Cli into a [CliParsed](struct.CliParsed.html).
    pub fn parse(&self) -> Result<CliParsed, ParseCliError> {
        Ok(CliParsed {
            pattern: self.pattern.clone(),
            temp_dir: self.temp_dir.as_ref().map(|d| PathBuf::from(d)),
            output: PathBuf::from(&self.output),
            mode: SelectionMode::from_string(
                &self.mode.as_ref().unwrap_or(&"outlier-3.0".to_string()),
            )
            .unwrap(),
            background: BackgroundMode::from_string(
                &self.background.as_ref().unwrap_or(&"first".to_string()),
            )
            .unwrap(),
            outlier: OutlierSelectionMode::from_string(
                &self.outlier.as_ref().unwrap_or(&"extreme".to_string()),
            )
            .unwrap(),
            compression: Compression::from_string(
                &self.compression.as_ref().unwrap_or(&"gzip".to_string()),
            )
            .unwrap(),
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
            debug: self.debug,
        })
    }
}

/// Parsed command line arguments.
#[allow(dead_code)]
#[derive(Debug)]
pub struct CliParsed {
    /// File search pattern
    pub pattern: String,
    /// Temp directory. Uses system temp directory if `None`.
    pub temp_dir: Option<PathBuf>,
    /// Path of the final output image.
    pub output: PathBuf,
    /// Pixel selection mode.
    pub mode: SelectionMode,
    /// Outlier selection mode in case more than one outlier is found (first|last|extreme|average). Optional, default 'extreme'.
    pub outlier: OutlierSelectionMode,
    /// Background pixel selection mode.
    pub background: BackgroundMode,
    /// Compression mode for time slices (gzip|zlib|deflate). Optional, default 'gzip'.
    pub compression: Compression,
    /// Output image quality for JPG files, in percent. Optional, default '95'.
    pub quality: u8,
    /// Print debug information (i.e. parsed cmd parameters).
    pub debug: bool,
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCliError(String);

impl fmt::Display for ParseCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
