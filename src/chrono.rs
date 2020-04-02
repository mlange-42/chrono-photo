//! Processes time-sliced data produced by [`TimeSlicer`](./time_slice/struct.TimeSlicer.html).
use crate::img_stream::PixelInputStream;
use image::flat::SampleLayout;
use std::path::PathBuf;

pub struct ChronoProcessor {}

impl ChronoProcessor {
    pub fn process(layout: &SampleLayout, files: &[PathBuf]) -> std::io::Result<Vec<u8>> {
        let channels = layout.width_stride;
        let buffer = vec![0; layout.height as usize * layout.height_stride];

        for file in files {
            let mut stream = PixelInputStream::new(file)?;
            let mut row = Vec::new();
            while let Some(num_bytes) = stream.read_chunk(&mut row) {
                row.clear();
            }
        }

        Ok(buffer)
    }
}
