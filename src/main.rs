use chrono_photo::cli::Cli;
use chrono_photo::img_stream::{ImageStream, PixelInputStream, PixelOutputStream};
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    //let args = Cli::from_args().parse().unwrap();

    let pattern = "test_data/*.png"; //args.pattern.unwrap();
    let img_stream = ImageStream::from_pattern(&pattern).expect("Error processing pattern");
    let out_file = "test_data/temp.bin";

    let mut layout: Option<image::flat::SampleLayout> = None;
    let mut count = 0;
    let mut out_stream = PixelOutputStream::new(PathBuf::from(out_file)).unwrap();
    for img in img_stream {
        let dyn_img = img.unwrap();
        let pix = dyn_img.as_flat_samples_u8().unwrap();
        match layout {
            Some(lay) => {
                if pix.layout != lay {
                    panic!("Image layout does not fit!");
                }
            }
            None => layout = Some(pix.layout),
        }
        out_stream.write_chunk(pix.samples).unwrap();
        count += 1;
    }
    out_stream.close().unwrap();
    if count == 0 {
        println!("WARNING: no images found for pattern {}", pattern);
    } else {
        println!("Processed {} images", count);
    }

    let mut reader = PixelInputStream::new(PathBuf::from(out_file)).unwrap();

    let mut buff = Vec::new();
    while let Some(size) = reader.read_chunk(&mut buff) {
        buff.clear();
    }
}
