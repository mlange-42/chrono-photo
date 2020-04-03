//! Lists files by pattern
extern crate glob;

use std::collections::VecDeque;
use std::path::PathBuf;

/// Lists files by searching for a file pattern.
#[derive(Debug)]
pub struct FileLister {
    pattern: String,
}

impl FileLister {
    /// Creates a new lister from a pattern.
    pub fn new(pattern: &str) -> Self {
        FileLister {
            pattern: pattern.to_string(),
        }
    }
    /// Lists all files that match this lister's pattern.
    pub fn list_files<'a>(&self) -> Result<VecDeque<PathBuf>, glob::PatternError> {
        // TODO Return an iterator instead of a vector. Having problems with "size not known at compile time".
        let paths: glob::Paths = glob::glob(&self.pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_file())
            .map(|p| p.unwrap())
            .collect();
        Ok(vec)
    }
}

#[cfg(test)]
mod test {
    use crate::flist::FileLister;

    #[test]
    fn parse_pattern() {
        let pattern = "test_data/*.txt";
        let lister = FileLister::new(&pattern);

        let _list = lister.list_files().expect("Error processing pattern");
    }
}
