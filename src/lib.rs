pub mod chrono;
pub mod cli;
pub mod flist;
pub mod img_stream;
pub mod time_slice;

use std::fmt;

pub trait EnumFromString {
    /// Parses a string to an `enum`.
    fn from_string(str: &str) -> Result<Self, ParseEnumError>
    where
        Self: std::marker::Sized;
}

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError(String);

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
