use chrono_photo::chrono::ChronoProcessor;
use chrono_photo::cli::{Cli, CliParsed};
use chrono_photo::flist::FrameRange;
//use chrono_photo::options::{BackgroundMode, OutlierSelectionMode, SelectionMode, Threshold};
use chrono_photo::slicer::{SliceLength, TimeSliceError, TimeSlicer};
use chrono_photo::streams::{Compression, ImageStream};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use std::cmp;
use std::fs::File;
use std::option::Option::Some;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

fn main() {
    let start = Instant::now();

    /*let mut args = CliParsed {
        pattern: "test_data/generated/image-*.jpg".to_string(),
        frames: Some(FrameRange::new(None, Some(10), 1)),
        video_in: Some(FrameRange::new(Some(0), Some(5), 1)),
        video_out: Some(FrameRange::new(None, None, 1)),
        temp_dir: Some(PathBuf::from("test_data/temp")),
        output: PathBuf::from("test_data/out.jpg"),
        output_blend: None,
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
    let (temp_files, layout, image_count) = match to_time_slices(
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
        create_video(&args, &temp_files[..], &layout, image_count);
    } else {
        // Process to image
        create_frame(
            &args,
            &temp_files[..],
            &layout,
            image_count,
            None,
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

fn create_video(args: &CliParsed, files: &[PathBuf], layout: &SampleLayout, image_count: usize) {
    let video = &args
        .video_out
        .as_ref()
        .expect("Video frame range required (`--video-out`)");
    let frames = &args
        .video_in
        .as_ref()
        .expect("Per-frame frame range required (`--video-in`)");

    let v_lower = match video.start() {
        Some(start) => start,
        None => {
            if let Some(r) = frames.range() {
                -r + 1
            } else {
                0
            }
        }
    };

    let v_upper = match video.end() {
        Some(end) => end,
        None => image_count as i32,
    };

    let mut indices = Vec::new();
    let mut frame = v_lower;
    while frame < v_upper {
        let start = match frames.start() {
            Some(s) => cmp::max(0, frame + s),
            None => 0,
        };
        let end = match frames.end() {
            Some(e) => cmp::min(image_count as i32, frame + e),
            None => image_count as i32,
        };

        indices.clear();
        let mut f = start;
        while f < end {
            indices.push(f as usize);
            f += frames.step() as i32;
        }
        let (name, ext) = name_and_extension(&args.output)
            .expect(&format!("Unexpected format in {:?}", &args.output));
        let mut output = args
            .output
            .parent()
            .expect(&format!("Unexpected format in {:?}", &args.output))
            .to_path_buf();
        output.push(&format!("{}-{:05}.{}", name, frame - v_lower, ext));

        let out_blend = args.output_blend.as_ref().and_then(|out| {
            let (name, ext) =
                name_and_extension(&out).expect(&format!("Unexpected format in {:?}", &out));
            let mut output = args
                .output
                .parent()
                .expect(&format!("Unexpected format in {:?}", &out))
                .to_path_buf();
            output.push(&format!("{}-{:05}.{}", name, frame - v_lower, ext));
            Some(output)
        });

        print!(
            "Processing frame {}/{} -> ",
            frame - v_lower,
            v_upper - v_lower
        );
        create_frame(
            &args,
            &files,
            &layout,
            image_count,
            Some(&indices[..]),
            &output,
            &out_blend,
        );
        frame += video.step() as i32;
    }
}

fn name_and_extension(path: &PathBuf) -> Option<(String, String)> {
    let stem = path.file_stem();
    if stem.is_none() {
        return None;
    }
    let stem = stem.unwrap().to_str();
    if stem.is_none() {
        return None;
    }

    let ext = path.extension();
    if ext.is_none() {
        return None;
    }
    let ext = ext.unwrap().to_str();
    if ext.is_none() {
        return None;
    }

    Some((stem.unwrap().to_string(), ext.unwrap().to_string()))
}

fn create_frame(
    args: &CliParsed,
    files: &[PathBuf],
    layout: &SampleLayout,
    image_count: usize,
    image_indices: Option<&[usize]>,
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
        .process(
            &layout,
            &files,
            &args.slice,
            Some(image_count),
            image_indices,
        )
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
