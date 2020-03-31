//! Command-line interface for chrono-photo.
use core::fmt;
use structopt::StructOpt;

/// Raw command line arguments.
#[derive(StructOpt)]
#[structopt(name = "chrono-photo command line application")]
pub struct Cli {
    #[structopt(short, long)]
    file: Option<String>,
    #[structopt(short, long)]
    pattern: Option<String>,
}

impl Cli {
    pub fn parse(&self) -> Result<CliParsed, ParseCliError> {
        if self.file.is_none() && self.pattern.is_none() {
            return Err( ParseCliError("Missing required option: either specify `--file` for processing a video, or `--pattern` for processing a sequence of images".to_string()) );
        }

        Ok(CliParsed {
            file: self.file.clone(),
            pattern: self.pattern.clone(),
        })
    }
}

pub struct CliParsed {
    file: Option<String>,
    pattern: Option<String>,
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCliError(String);

impl fmt::Display for ParseCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
