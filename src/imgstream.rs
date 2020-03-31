//! Provides an image stream from a list of files or a (TODO) video file.
use crate::flist::FileLister;
use glob::PatternError;
use image;
use std::path::PathBuf;

pub struct ImageStream {
    files: Vec<PathBuf>,
}
impl ImageStream {
    pub fn from_pattern(pattern: &str) -> Result<Self, PatternError> {
        let lister = FileLister::new(&pattern);
        let files = lister.list_files()?;
        Ok(ImageStream { files })
    }
}
impl Iterator for ImageStream {
    type Item = image::DynamicImage;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::flist::FileLister;
    use crate::imgstream::ImageStream;

    #[test]
    fn iterate() {
        let pattern = "test_data/*.txt";
        let stream = ImageStream::from_pattern(&pattern).expect("Error processing pattern");
    }
}
