use chrono_photo::chrono::OutlierProcessor;
use chrono_photo::cli::{Cli, CliParsed};
use chrono_photo::flist::{FileLister, FrameRange};
use chrono_photo::options::SelectionMode;
//use chrono_photo::options::{BackgroundMode, Fade, OutlierSelectionMode, SelectionMode, Threshold};
use chrono_photo::shake::{Crop, ShakeAnalyzer};
use chrono_photo::simple::SimpleProcessor;
use chrono_photo::slicer::{SliceLength, TimeSliceError, TimeSlicer};
use chrono_photo::streams::{Compression, ImageStream};
use image::flat::SampleLayout;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs::File;
use std::option::Option::Some;
use std::path::PathBuf;
use std::time::Instant;
use std::{cmp, env, fs};
use structopt::StructOpt;

fn main() {
    let start = Instant::now();

    /*let args = CliParsed {
        pattern: "C:\\Data\\Private\\Photos\\2020-04-17_Fabian_Chrono\\images_001\\*.jpg"
            .to_string(),
        frames: None,
        video_in: None,
        video_out: None,
        temp_dir: Some(PathBuf::from("test_data/temp")),
        output: PathBuf::from("test_data/out.jpg"),
        output_blend: None,
        mode: SelectionMode::Outlier,
        threshold: Threshold::abs(0.05, 0.2),
        outlier: OutlierSelectionMode::Extreme,
        background: BackgroundMode::First,
        compression: Compression::GZip(6),
        quality: 98,
        slice: SliceLength::Rows(1),
        sample: None,
        threads: Some(1),
        video_threads: Some(1),
        fade: Fade::none(),
        weights: [0.0, 1.0, 1.0, 0.0],
        shake_reduction: Some(ShakeReduction::new(vec![(772, 971), (1109, 539)], 10, 20)),
        debug: true,
    };*/
    let args: Vec<String> = env::args().collect();
    let args: CliParsed = if args.len() == 2 && !args[1].starts_with('-') {
        let mut content = fs::read_to_string(&args[1]).expect(&format!(
            "Something went wrong reading the options file {:?}",
            &args[1]
        ));
        content = "chrono-photo ".to_string() + &content.replace("\r\n", " ").replace("\n", " ");
        let cli: Cli = content.parse().unwrap();
        cli.parse().unwrap()
    } else {
        Cli::from_args().parse().unwrap()
    };

    if args.debug {
        println!("{:#?}", args);
    }

    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Error building thread pool. Pool already built.");
    }

    let lister = FileLister::new(&args.pattern, &args.frames);
    let files = lister.files_vec().expect(&format!(
        "Unable to process search pattern {:?}",
        &args.pattern
    ));
    let shake = args.shake_reduction.as_ref().and_then(|red| {
        Some(
            ShakeAnalyzer {}
                .analyze(
                    &files[..],
                    red.anchors(),
                    red.anchor_radius(),
                    red.search_radius(),
                    true,
                )
                .expect("Shake analysis failed!"),
        )
    });
    let crop: Option<Vec<Crop>> = shake
        .as_ref()
        .and_then(|(offset, layout)| Crop::create(&offset[..], layout));

    if args.shake_reduction.is_some() {
        if crop.is_some() {
            println!("Camera shake detected. Images will be corrected.");
        } else {
            println!("No camera shake detected. Images will not be corrected.");
        }
    }

    if args.mode == SelectionMode::Outlier {
        run_outliers(args, &crop);
    } else {
        run_simple(args, &crop);
    }

    println!("Total time: {:?}", start.elapsed());
}

fn run_simple(mut args: CliParsed, crop: &Option<Vec<Crop>>) {
    let lister = FileLister::new(&args.pattern, &args.frames);
    let files = lister.files_vec().expect(&format!(
        "Unable to process search pattern {:?}",
        &args.pattern
    ));

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
        create_video_simple(&args, &files[..], &crop, args.video_threads);
    } else {
        // Process to image
        create_frame_simple(&args, &files[..], &crop, None, &args.output, true);
    }
}

fn run_outliers(mut args: CliParsed, crop: &Option<Vec<Crop>>) {
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

    // Convert to time slices and save to temp files
    let (temp_files, layout, image_count) = match to_time_slices(
        &args.pattern,
        crop,
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
        create_video(
            &args,
            &temp_files[..],
            &layout,
            image_count,
            args.video_threads,
        );
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
            true,
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
}

fn create_video(
    args: &CliParsed,
    files: &[PathBuf],
    layout: &SampleLayout,
    image_count: usize,
    threads: Option<usize>,
) {
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

    //let mut indices = Vec::new();
    //let mut frame = v_lower;

    let all_frames: Vec<_> = (0..((v_upper - v_lower) / video.step() as i32))
        .map(|i| i * video.step() as i32 + v_lower)
        .collect();
    //while frame < v_upper {

    let pool = match threads {
        Some(threads) => rayon::ThreadPoolBuilder::new().num_threads(threads),
        None => rayon::ThreadPoolBuilder::new(),
    }
    .build()
    .expect("Unable to build thread pool.");
    pool.install(|| {
        all_frames.par_iter().for_each(|frame| {
            let start = match frames.start() {
                Some(s) => {
                    let mut st = frame + s;
                    while st < 0 {
                        st += frames.step() as i32
                    }
                    cmp::max(st % frames.step() as i32, frame + s)
                }
                None => 0,
            };
            let end = match frames.end() {
                Some(e) => cmp::min(
                    image_count as i32 + (frame + e) % frames.step() as i32 - frames.step() as i32,
                    frame + e,
                ),
                None => image_count as i32,
            };

            //indices.clear();
            let mut indices = Vec::new();
            let mut f = start;
            while f < end {
                indices.push(f as usize);
                f += frames.step() as i32;
            }
            if !indices.is_empty() {
                let (name, ext) = name_and_extension(&args.output)
                    .expect(&format!("Unexpected format in {:?}", &args.output));
                let mut output = args
                    .output
                    .parent()
                    .expect(&format!("Unexpected format in {:?}", &args.output))
                    .to_path_buf();
                output.push(&format!("{}-{:05}.{}", name, frame - v_lower, ext));

                let out_blend = args.output_blend.as_ref().and_then(|out| {
                    let (name, ext) = name_and_extension(&out)
                        .expect(&format!("Unexpected format in {:?}", &out));
                    let mut output = args
                        .output
                        .parent()
                        .expect(&format!("Unexpected format in {:?}", &out))
                        .to_path_buf();
                    output.push(&format!("{}-{:05}.{}", name, frame - v_lower, ext));
                    Some(output)
                });

                println!(
                    "Processing frame {}/{} -> ",
                    frame - v_lower,
                    v_upper - v_lower
                );
                create_frame(
                    &args,
                    &files,
                    layout,
                    image_count,
                    Some(&indices[..]),
                    &output,
                    &out_blend,
                    false,
                );
            } else {
                println!("Skipping frame {}/{}", frame - v_lower, v_upper - v_lower);
            }

            //frame += video.step() as i32;
        });
    });
}

fn create_video_simple(
    args: &CliParsed,
    files: &[PathBuf],
    crop: &Option<Vec<Crop>>,
    threads: Option<usize>,
) {
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
        None => files.len() as i32,
    };

    //let mut indices = Vec::new();
    //let mut frame = v_lower;

    let all_frames: Vec<_> = (0..((v_upper - v_lower) / video.step() as i32))
        .map(|i| i * video.step() as i32 + v_lower)
        .collect();
    //while frame < v_upper {
    let pool = match threads {
        Some(threads) => rayon::ThreadPoolBuilder::new().num_threads(threads),
        None => rayon::ThreadPoolBuilder::new(),
    }
    .build()
    .expect("Unable to build thread pool.");
    pool.install(|| {
        all_frames.par_iter().for_each(|frame| {
            let start = match frames.start() {
                Some(s) => {
                    let mut st = frame + s;
                    while st < 0 {
                        st += frames.step() as i32
                    }
                    cmp::max(st % frames.step() as i32, frame + s)
                }
                None => 0,
            };
            let end = match frames.end() {
                Some(e) => cmp::min(
                    files.len() as i32 + (frame + e) % frames.step() as i32 - frames.step() as i32,
                    frame + e,
                ),
                None => files.len() as i32,
            };

            //indices.clear();
            let mut indices = Vec::new();
            let mut f = start;
            while f < end {
                indices.push(f as usize);
                f += frames.step() as i32;
            }
            if !indices.is_empty() {
                let (name, ext) = name_and_extension(&args.output)
                    .expect(&format!("Unexpected format in {:?}", &args.output));

                let mut output = args
                    .output
                    .parent()
                    .expect(&format!("Unexpected format in {:?}", &args.output))
                    .to_path_buf();
                output.push(&format!("{}-{:05}.{}", name, frame - v_lower, ext));

                println!(
                    "Processing frame {}/{} -> ",
                    frame - v_lower,
                    v_upper - v_lower
                );
                create_frame_simple(&args, &files, crop, Some(&indices[..]), &output, false);
            } else {
                println!("Skipping frame {}/{}", frame - v_lower, v_upper - v_lower);
            }

            //frame += video.step() as i32;
        });
    });
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
    show_progress: bool,
) {
    // Process time slices
    let processor = OutlierProcessor::new(
        args.threshold.clone(),
        args.background.clone(),
        args.outlier.clone(),
        args.weights.clone(),
        args.fade.clone(),
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
            show_progress,
        )
        .unwrap();

    if show_progress {
        println!("Saving output... ");
    }
    save_image(&buff, &layout, &output, args.quality);
    if let Some(out) = &output_blend {
        save_image(&is_outlier, &layout, &out, args.quality);
    }
}

fn create_frame_simple(
    args: &CliParsed,
    files: &[PathBuf],
    crop: &Option<Vec<Crop>>,
    image_indices: Option<&[usize]>,
    output: &PathBuf,
    show_progress: bool,
) {
    // Process time slices
    let processor = SimpleProcessor::new(
        args.weights.clone(),
        args.fade.clone(),
        args.mode == SelectionMode::Darker,
    );
    let (buff, layout) = processor
        .process(files, crop, image_indices, show_progress)
        .unwrap();

    if show_progress {
        println!("Saving output... ");
    }
    save_image(&buff, &layout, &output, args.quality);
}

fn save_image(buffer: &[u8], layout: &SampleLayout, out_path: &PathBuf, quality: u8) {
    let ext = out_path
        .extension()
        .expect("Expects an extension for output file to determine image format.")
        .to_str()
        .expect("Expects Unicode encoding for output file.")
        .to_lowercase();

    let parent = out_path
        .parent()
        .expect(&format!("Not a valid output path: {:?}", out_path));
    if !parent.is_dir() {
        std::fs::create_dir(parent)
            .expect(&format!("Unable to create output directory {:?}", parent));
    }

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
    crop: &Option<Vec<Crop>>,
    is_16bit: bool,
    frames: &Option<FrameRange>,
    temp_path: &PathBuf,
    compression: &Compression,
    slices: &SliceLength,
) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
    let images =
        ImageStream::from_pattern(image_pattern, frames).expect("Error processing pattern");
    if is_16bit {
        TimeSlicer::new_16bit().write_time_slices(
            images,
            crop,
            temp_path.clone(),
            compression,
            slices,
        )
    } else {
        TimeSlicer::new_8bit().write_time_slices(
            images,
            crop,
            temp_path.clone(),
            compression,
            slices,
        )
    }
}
