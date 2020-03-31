//! Command-line interface for chrono-photo.
use core::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

/// Raw command line arguments.
#[derive(StructOpt)]
#[structopt(name = "chrono-photo command line application")]
pub struct Cli {
    #[structopt(short, long)]
    pattern: String,
    #[structopt(short, long, name = "temp-dir")]
    temp_dir: Option<String>,
}

impl Cli {
    pub fn parse(&self) -> Result<CliParsed, ParseCliError> {
        Ok(CliParsed {
            pattern: self.pattern.clone(),
            temp_dir: self.temp_dir.as_ref().map(|d| PathBuf::from(d)),
        })
    }
}

#[allow(dead_code)]
pub struct CliParsed {
    pub pattern: String,
    pub temp_dir: Option<PathBuf>,
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCliError(String);

impl fmt::Display for ParseCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
