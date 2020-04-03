//! Provides an image stream from a list of files, or a (TODO) video file.
use crate::flist::FileLister;
use crate::{EnumFromString, ParseEnumError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use glob::PatternError;
use image;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Compression {
    GZip,
    ZLib,
    Deflate,
}
impl EnumFromString for Compression {
    fn from_string(str: &str) -> Result<Self, ParseEnumError>
    where
        Self: std::marker::Sized,
    {
        match str {
            "gzip" => Ok(Compression::GZip),
            "zlib" => Ok(Compression::ZLib),
            "deflate" => Ok(Compression::Deflate),
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
impl ImageStream {
    /// The number of images in this stream
    pub fn len(&self) -> usize {
        self.files.len()
    }
}

pub struct PixelOutputStream {
    path: PathBuf,
    stream: BufWriter<std::fs::File>,
    compression: Compression,
}
impl PixelOutputStream {
    pub fn new(path: &PathBuf, compression: Compression) -> std::io::Result<Self> {
        let stream = BufWriter::new(File::create(path)?);
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
        match self.compression {
            Compression::GZip => {
                let mut e = GzEncoder::new(Vec::new(), flate2::Compression::default());
                e.write_all(bytes)?;
                let compressed = &e.finish()?;
                self.stream
                    .write_u32::<BigEndian>(compressed.len() as u32)?;
                self.stream.write_all(compressed)?;
                self.stream.flush()?;
                Ok(compressed.len())
            }
            Compression::ZLib => {
                let mut e = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
                e.write_all(bytes)?;
                let compressed = &e.finish()?;
                self.stream
                    .write_u32::<BigEndian>(compressed.len() as u32)?;
                self.stream.write_all(compressed)?;
                self.stream.flush()?;
                Ok(compressed.len())
            }
            Compression::Deflate => {
                let mut e = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
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
        match self.compression {
            Compression::GZip => {
                let mut d = GzDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
            Compression::ZLib => {
                let mut d = ZlibDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
            Compression::Deflate => {
                let mut d = DeflateDecoder::new(&compressed[..]);
                let size = d.read_to_end(out).unwrap();
                Some(size)
            }
        }
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
        let mut stream = PixelOutputStream::new(&PathBuf::from("test_data/temp.bin")).unwrap();

        stream.write_chunk(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        /*
        for img in stream {
            println!("{:?}", img.unwrap().color());
        }*/
    }
}
