//! Lists files by pattern
extern crate glob;

use crate::ParseOptionError;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;

/// A frame range, defined by start, end (both optional) and step.
#[derive(Clone, Debug)]
pub struct FrameRange {
    start: Option<i32>,
    end: Option<i32>,
    step: u32,
}
impl FrameRange {
    /// Creates a new FrameRange
    pub fn new(start: Option<i32>, end: Option<i32>, step: u32) -> Self {
        FrameRange { start, end, step }
    }
    /// Creates a new FrameRange without start and end, and with a step of 1.
    pub fn empty() -> Self {
        FrameRange {
            start: None,
            end: None,
            step: 1,
        }
    }
    /// The start frame
    pub fn start(&self) -> Option<i32> {
        self.start
    }
    /// The end frame (exclusive)
    pub fn end(&self) -> Option<i32> {
        self.end
    }
    /// The step
    pub fn step(&self) -> u32 {
        self.step
    }
    /// The total number of frames, irrespective of step
    pub fn range(&self) -> Option<i32> {
        self.start.and_then(|s| self.end.and_then(|e| Some(e - s)))
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
            step: values[2].unwrap_or(1) as u32,
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
    pub fn new(pattern: &str, frames: &Option<FrameRange>) -> Self {
        FileLister {
            pattern: pattern.to_string(),
            frames: frames.clone(),
        }
    }
    /// Lists all files that match this lister's pattern.
    pub fn files_vecdeque<'a>(&self) -> Result<VecDeque<PathBuf>, glob::PatternError> {
        // TODO Return an iterator instead of a vector. Having problems with "size not known at compile time".
        let paths: glob::Paths = glob::glob(&self.pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_file())
            .map(|p| p.unwrap());
        match &self.frames {
            Some(fr) => Ok(vec
                .take(fr.end.unwrap_or(std::i32::MAX) as usize)
                .skip(fr.start.unwrap_or(0) as usize)
                .enumerate()
                .filter_map(|(i, p)| {
                    if i as u32 % fr.step == 0 {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect()),
            None => Ok(vec.collect()),
        }
    }

    /// Lists all files that match this lister's pattern.
    pub fn files_vec<'a>(&self) -> Result<Vec<PathBuf>, glob::PatternError> {
        // TODO Return an iterator instead of a vector. Having problems with "size not known at compile time".
        let paths: glob::Paths = glob::glob(&self.pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_file())
            .map(|p| p.unwrap());
        match &self.frames {
            Some(fr) => Ok(vec
                .take(fr.end.unwrap_or(std::i32::MAX) as usize)
                .skip(fr.start.unwrap_or(0) as usize)
                .enumerate()
                .filter_map(|(i, p)| {
                    if i as u32 % fr.step == 0 {
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
        let lister = FileLister::new(&pattern, &None);

        let _list = lister.files_vecdeque().expect("Error processing pattern");
    }
}
