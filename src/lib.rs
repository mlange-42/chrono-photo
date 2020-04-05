pub mod chrono;
pub mod cli;
pub mod color;
pub mod flist;
pub mod img_stream;
pub mod options;
pub mod time_slice;

use std::fmt;

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError(String);

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Error type for failed parsing of `String`s to a CLI option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptionError(String);

impl fmt::Display for ParseOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
