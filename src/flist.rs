//! Lists files by pattern
extern crate glob;

use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileLister {
    pattern: String,
}

impl FileLister {
    pub fn new(pattern: &str) -> Self {
        FileLister {
            pattern: pattern.to_string(),
        }
    }

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

        let list = lister.list_files().expect("Error processing pattern");

        for entry in list {
            //println!("{:?}", entry);
        }
    }
}
