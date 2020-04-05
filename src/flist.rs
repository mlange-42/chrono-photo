//! Lists files by pattern
extern crate glob;

use crate::ParseOptionError;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct FrameRange {
    start: Option<usize>,
    end: Option<usize>,
    step: Option<usize>,
}
impl FrameRange {
    pub fn new(start: Option<usize>, end: Option<usize>, step: Option<usize>) -> Self {
        FrameRange { start, end, step }
    }
}
impl FromStr for FrameRange {
    type Err = ParseOptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(ParseOptionError(format!(
                "Option --frames expects 3 elements: start/end/step, {} were suppied",
                parts.len()
            )));
        }
        let mut values = vec![None; 3];
        for (i, s) in parts.iter().enumerate() {
            if *s == "." {
                values[i] = None;
            } else {
                match s.parse() {
                    Ok(v) => values[i] = Some(v),
                    Err(err) => {
                        return Err(ParseOptionError(format!(
                            "Can't parse element {} in option --frames (start/end/step), got '{}'. [{:?}]",
                            i, s, err
                        )))
                    }
                }
            }
        }
        Ok(FrameRange {
            start: values[0],
            end: values[1],
            step: values[2],
        })
    }
}

/// Lists files by searching for a file pattern.
#[derive(Debug)]
pub struct FileLister {
    pattern: String,
    frames: Option<FrameRange>,
}

impl FileLister {
    /// Creates a new lister from a pattern.
    pub fn new(pattern: &str, frames: Option<FrameRange>) -> Self {
        FileLister {
            pattern: pattern.to_string(),
            frames,
        }
    }
    /// Lists all files that match this lister's pattern.
    pub fn list_files<'a>(&self) -> Result<VecDeque<PathBuf>, glob::PatternError> {
        // TODO Return an iterator instead of a vector. Having problems with "size not known at compile time".
        let paths: glob::Paths = glob::glob(&self.pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_file())
            .map(|p| p.unwrap());
        match &self.frames {
            Some(fr) => Ok(vec
                .take(fr.end.unwrap_or(std::usize::MAX))
                .skip(fr.start.unwrap_or(0))
                .enumerate()
                .filter_map(|(i, p)| {
                    if i % fr.step.unwrap_or(1) == 0 {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect()),
            None => Ok(vec.collect()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::flist::FileLister;

    #[test]
    fn parse_pattern() {
        let pattern = "test_data/*.txt";
        let lister = FileLister::new(&pattern, None);

        let _list = lister.list_files().expect("Error processing pattern");
    }
}
