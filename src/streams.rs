//! Provides an image stream from a list of files, or a (TODO) video file.
use crate::flist::{FileLister, FrameRange};
use crate::ParseEnumError;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use glob::PatternError;
use image;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Compression {
    GZip(u32),
    ZLib(u32),
    Deflate(u32),
}
impl FromStr for Compression {
    type Err = ParseEnumError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split('/').collect();
        let str = parts
            .get(0)
            .expect(&format!("Unexpected format in compression {}", str));
        let level = if let Some(num) = parts.get(1) {
            num.parse()
                .expect(&format!("Unable to parse compression level in {}", num))
        } else {
            6
        };

        match str {
            &"gzip" => Ok(Compression::GZip(level)),
            &"zlib" => Ok(Compression::ZLib(level)),
            &"deflate" => Ok(Compression::Deflate(level)),
            _ => Err(ParseEnumError(format!(
                "Not a compression: {}. Must be one of (gzip|zlib|deflate)",
                str
            ))),
        }
    }
}

/// Provides a stream of images from a file search pattern.
pub struct ImageStream {
    files: VecDeque<PathBuf>,
}
impl ImageStream {
    /// Creates an ImageStream from a file search pattern.
    pub fn from_pattern(pattern: &str, frames: &Option<FrameRange>) -> Result<Self, PatternError> {
        let lister = FileLister::new(&pattern, frames);
        let files = lister.files_vecdeque()?;
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
impl ImageStream {
    /// The number of images in this stream
    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn skip(&mut self) -> Option<()> {
        if self.files.is_empty() {
            None
        } else {
            let _path = self.files.pop_front().unwrap();
            Some(())
        }
    }
}

pub struct PixelOutputStream {
    path: PathBuf,
    stream: BufWriter<std::fs::File>,
    compression: Compression,
}
impl PixelOutputStream {
    pub fn new(path: &PathBuf, compression: Compression, append: bool) -> std::io::Result<Self> {
        let file = if append {
            OpenOptions::new().write(true).append(true).open(path)?
        } else {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?
        };
        let stream = BufWriter::new(file);
        let stream = PixelOutputStream {
            path: path.clone(),
            stream,
            compression,
        };
        Ok(stream)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn write_chunk(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        // TODO: make generic
        match self.compression {
            Compression::GZip(level) => {
                let mut e = GzEncoder::new(Vec::new(), flate2::Compression::new(level));
                e.write_all(bytes)?;
                let compressed = &e.finish()?;
                self.stream
                    .write_u32::<BigEndian>(compressed.len() as u32)?;
                self.stream.write_all(compressed)?;
                self.stream.flush()?;
                Ok(compressed.len())
            }
            Compression::ZLib(level) => {
                let mut e = ZlibEncoder::new(Vec::new(), flate2::Compression::new(level));
                e.write_all(bytes)?;
                let compressed = &e.finish()?;
                self.stream
                    .write_u32::<BigEndian>(compressed.len() as u32)?;
                self.stream.write_all(compressed)?;
                self.stream.flush()?;
                Ok(compressed.len())
            }
            Compression::Deflate(level) => {
                let mut e = DeflateEncoder::new(Vec::new(), flate2::Compression::new(level));
                e.write_all(bytes)?;
                let compressed = &e.finish()?;
                self.stream
                    .write_u32::<BigEndian>(compressed.len() as u32)?;
                self.stream.write_all(compressed)?;
                self.stream.flush()?;
                Ok(compressed.len())
            }
        }
    }
    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

pub struct PixelInputStream {
    stream: BufReader<File>,
    compression: Compression,
}
impl PixelInputStream {
    pub fn new(file: &PathBuf, compression: Compression) -> std::io::Result<Self> {
        let f = File::open(file)?;
        //let d = GzDecoder::new(f);
        let stream = PixelInputStream {
            stream: BufReader::new(f),
            compression,
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
        // TODO: Make generic
        match self.compression {
            Compression::GZip(_) => {
                let mut d = GzDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
            Compression::ZLib(_) => {
                let mut d = ZlibDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
            Compression::Deflate(_) => {
                let mut d = DeflateDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
        }
    }

    pub fn skip_chunk(&mut self) -> Option<usize> {
        let len = match self.stream.read_u32::<BigEndian>() {
            Ok(l) => l,
            Err(err) => match err.kind() {
                std::io::ErrorKind::UnexpectedEof => return None,
                _ => panic!(err),
            },
        };
        if let Err(err) = self.stream.seek(SeekFrom::Current(len as i64)) {
            match err.kind() {
                std::io::ErrorKind::UnexpectedEof => return None,
                _ => {}
            }
        }
        Some(0)
    }
}

#[cfg(test)]
mod test {
    use crate::streams::ImageStream;

    #[test]
    fn iterate() {
        let pattern = "test_data/*.png";
        let _stream = ImageStream::from_pattern(&pattern, &None).expect("Error processing pattern");
        /*
        for img in stream {
            println!("{:?}", img.unwrap().color());
        }*/
    }
}
