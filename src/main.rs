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
        //is_16bit: true,
        frames: Some(FrameRange::new(None, None, Some(1))),
        temp_dir: Some(PathBuf::from("test_data/temp")),
        output: PathBuf::from("test_data/out.jpg"),
        output_blend: Some(PathBuf::from("test_data/out-debug.png")),
        mode: SelectionMode::Outlier,
        threshold: Threshold::abs(0.05, 0.2),
        outlier: OutlierSelectionMode::Extreme,
        background: BackgroundMode::Random,
        compression: Compression::GZip(6),
        quality: 98,
        slice: SliceLength::Rows(1),
        sample: None,
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
        false,
        &args.frames,
        &args.temp_dir.as_ref().unwrap(),
        &args.compression,
        &args.slice,
    ) {
        Ok(fls) => fls,
        Err(err) => {
            println!("{:?}", err.to_string());
            return;
        }
    };

    // Process to video or image
    if args.video_in.is_some() || args.video_out.is_some() {
        // Fill missing video range
        if args.video_in.is_some() {
            if args.video_out.is_none() {
                args.video_out = Some(FrameRange::empty());
            }
        } else if args.video_out.is_some() {
            args.video_in = Some(FrameRange::empty());
        }
        // Process to video
        create_video(&args, &temp_files[..], &layout, size_hint);
    } else {
        // Process to image
        create_frame(
            &args,
            &temp_files[..],
            &layout,
            size_hint,
            &args.output,
            &args.output_blend,
        );
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

fn create_video(args: &CliParsed, files: &[PathBuf], layout: &SampleLayout, size_hint: usize) {}
fn create_frame(
    args: &CliParsed,
    files: &[PathBuf],
    layout: &SampleLayout,
    size_hint: usize,
    output: &PathBuf,
    output_blend: &Option<PathBuf>,
) {
    // Process time slices
    let processor = ChronoProcessor::new(
        args.mode.clone(),
        args.threshold.clone(),
        args.background.clone(),
        args.outlier.clone(),
        args.compression.clone(),
        args.sample.clone(),
    );
    let (buff, is_outlier) = processor
        .process(&layout, &files, None, &args.slice, Some(size_hint))
        .unwrap();

    println!("Saving output... ");
    save_image(&buff, &layout, &output, args.quality);
    if let Some(out) = &output_blend {
        save_image(&is_outlier, &layout, &out, args.quality);
    }
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
    is_16bit: bool,
    frames: &Option<FrameRange>,
    temp_path: &PathBuf,
    compression: &Compression,
    slices: &SliceLength,
) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
    let images =
        ImageStream::from_pattern(image_pattern, frames).expect("Error processing pattern");
    if is_16bit {
        TimeSlicer::new_16bit().write_time_slices(images, temp_path.clone(), compression, slices)
    } else {
        TimeSlicer::new_8bit().write_time_slices(images, temp_path.clone(), compression, slices)
    }
}
