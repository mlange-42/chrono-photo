use chrono_photo::chrono::ChronoProcessor;
use chrono_photo::cli::{Cli, CliParsed};
use chrono_photo::flist::FrameRange;
//use chrono_photo::options::{BackgroundMode, OutlierSelectionMode, SelectionMode, Threshold};
use chrono_photo::slicer::{SliceLength, TimeSliceError, TimeSlicer};
use chrono_photo::streams::{Compression, ImageStream};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

fn main() {
    let start = Instant::now();

    /*let mut args = CliParsed {
        pattern: "test_data/generated/image-*.jpg".to_string(),
        frames: Some(FrameRange::new(None, None, Some(2))),
        temp_dir: Some(PathBuf::from("test_data/temp")),
        output: PathBuf::from("test_data/out.jpg"),
        output_blend: Some(PathBuf::from("test_data/out-debug.png")),
        mode: SelectionMode::Outlier,
        threshold: Threshold::abs(0.05, 0.2),
        outlier: OutlierSelectionMode::Extreme,
        background: BackgroundMode::Random,
        compression: Compression::GZip(6),
        quality: 98,
        slice: SliceLength::Count(100),
        debug: true,
    };*/

    let mut args: CliParsed = Cli::from_args().parse().unwrap();

    // Determine temp directory
    if args.temp_dir.is_none() {
        let mut dir = std::env::temp_dir();
        dir.push("chrono-photo");
        args.temp_dir = Some(dir);
    }
    let temp_dir = args.temp_dir.as_ref().unwrap();
    print!("Temp directory: {:?}", temp_dir);

    // Create temp dir (only 1 level of creation depth)
    if !temp_dir.is_dir() {
        std::fs::create_dir(temp_dir)
            .expect(&format!("Unable to create temp directory {:?}", temp_dir));
        println!(" -> created.");
    } else {
        println!();
    }

    if args.debug {
        println!("{:#?}", args);
    }

    // Convert to time slices and save to temp files
    let (temp_files, layout, size_hint) = match to_time_slices(
        &args.pattern,
        args.frames,
        &args.temp_dir.unwrap(),
        &args.compression,
        &args.slice,
    ) {
        Ok(fls) => fls,
        Err(err) => {
            println!("{:?}", err.to_string());
            return;
        }
    };

    // Process time slices
    let processor = ChronoProcessor::new(
        args.mode,
        args.threshold,
        args.background,
        args.outlier,
        args.compression,
    );
    let (buff, is_outlier) = processor
        .process(&layout, &temp_files[..], &args.slice, Some(size_hint))
        .unwrap();

    println!("Saving output... ");
    save_image(&buff, &layout, &args.output, args.quality);
    if let Some(out) = &args.output_blend {
        save_image(&is_outlier, &layout, &out, args.quality);
    }

    // Delete temp file
    println!("Deleting {} time slices", temp_files.len());
    let bar = ProgressBar::new(temp_files.len() as u64);
    for file in &temp_files {
        bar.inc(1);
        match std::fs::remove_file(file) {
            Ok(()) => {}
            Err(err) => println!("Unable to delete file {:?}: {}", file, err.to_string()),
        }
    }
    bar.finish_and_clear();

    println!("Total time: {:?}", start.elapsed());
}

fn save_image(buffer: &[u8], layout: &SampleLayout, out_path: &PathBuf, quality: u8) {
    let ext = out_path
        .extension()
        .expect("Expects an extension for output file to determine image format.")
        .to_str()
        .expect("Expects Unicode encoding for output file.")
        .to_lowercase();

    if ext == "jpg" || ext == "jpeg" {
        let mut file = File::create(&out_path)
            .expect(&format!("Unable to create output file {:?}.", &out_path));
        let mut enc = image::jpeg::JPEGEncoder::new_with_quality(&mut file, quality);
        enc.encode(
            &buffer,
            layout.width,
            layout.height,
            if layout.width_stride == 4 {
                image::ColorType::Rgba8
            } else {
                image::ColorType::Rgb8
            },
        )
        .expect(&format!("Unable to write output file {:?}.", &out_path));
    } else {
        image::save_buffer(
            &out_path,
            &buffer,
            layout.width,
            layout.height,
            if layout.width_stride == 4 {
                image::ColorType::Rgba8
            } else {
                image::ColorType::Rgb8
            },
        )
        .expect(&format!("Unable to save output file {:?}", &out_path));
    }
}

fn to_time_slices(
    image_pattern: &str,
    frames: Option<FrameRange>,
    temp_path: &PathBuf,
    compression: &Compression,
    slices: &SliceLength,
) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
    let images =
        ImageStream::from_pattern(image_pattern, frames).expect("Error processing pattern");
    TimeSlicer::write_time_slices(images, temp_path.clone(), compression, slices)
}
