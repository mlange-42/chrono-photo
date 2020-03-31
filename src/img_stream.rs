//! Provides an image stream from a list of files or a (TODO) video file.
use crate::flist::FileLister;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use glob::PatternError;
use image;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub struct ImageStream {
    files: VecDeque<PathBuf>,
}
impl ImageStream {
    pub fn from_pattern(pattern: &str) -> Result<Self, PatternError> {
        let lister = FileLister::new(&pattern);
        let files = lister.list_files()?;
        Ok(ImageStream { files })
    }
}
impl Iterator for ImageStream {
    type Item = image::ImageResult<image::DynamicImage>;

    fn next(&mut self) -> Option<image::ImageResult<image::DynamicImage>> {
        if self.files.is_empty() {
            None
        } else {
            let path = self.files.pop_front().unwrap();
            Some(image::open(&path))
        }
    }
}

pub struct PixelOutputStream {
    path: PathBuf,
    stream: BufWriter<std::fs::File>,
}
impl PixelOutputStream {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let stream = BufWriter::new(File::create(&path)?);
        let stream = PixelOutputStream { path, stream };
        Ok(stream)
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn write_chunk(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(bytes)?;
        let compressed = &e.finish()?;
        //println!("Compressed {} to {}", bytes.len(), compressed.len());
        self.stream
            .write_u32::<BigEndian>(compressed.len() as u32)?;
        self.stream.write_all(compressed)
    }
    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

pub struct PixelInputStream {
    stream: BufReader<File>,
}
impl PixelInputStream {
    pub fn new(file: PathBuf) -> std::io::Result<Self> {
        let f = File::open(file)?;
        //let d = GzDecoder::new(f);
        let stream = PixelInputStream {
            stream: BufReader::new(f),
        };
        Ok(stream)
    }
    pub fn read_chunk(&mut self, out: &mut Vec<u8>) -> Option<usize> {
        let len = match self.stream.read_u32::<BigEndian>() {
            Ok(l) => l,
            Err(err) => match err.kind() {
                std::io::ErrorKind::UnexpectedEof => return None,
                _ => panic!(err),
            },
        };
        let mut compressed = vec![0_u8; len as usize];
        if let Err(err) = self.stream.read_exact(&mut compressed) {
            match err.kind() {
                std::io::ErrorKind::UnexpectedEof => return None,
                _ => {}
            }
        }
        let mut d = GzDecoder::new(&compressed[..]);
        let size = d.read_to_end(out).unwrap();
        //println!("Decompressed {} to {}", compressed.len(), size);
        Some(size)
    }
}

#[cfg(test)]
mod test {
    use crate::img_stream::{ImageStream, PixelOutputStream};
    use std::path::PathBuf;

    #[test]
    fn iterate() {
        let pattern = "test_data/*.png";
        let stream = ImageStream::from_pattern(&pattern).expect("Error processing pattern");
        /*
        for img in stream {
            println!("{:?}", img.unwrap().color());
        }*/
    }
    #[test]
    fn pixel_stream() {
        let mut stream = PixelOutputStream::new(PathBuf::from("test_data/temp.bin")).unwrap();

        stream.write_chunk(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        /*
        for img in stream {
            println!("{:?}", img.unwrap().color());
        }*/
    }
}
