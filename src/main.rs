use chrono_photo::chrono::ChronoProcessor;
use chrono_photo::cli::Cli;
use chrono_photo::img_stream::ImageStream;
use chrono_photo::time_slice::{TimeSliceError, TimeSlicer};
use image::flat::SampleLayout;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    /*let mut args = CliParsed {
        pattern: "test_data/TestImage-*.png".to_string(),
        temp_dir: Some(PathBuf::from("test_data/temp")),
        output: PathBuf::from("test_data/out.png"),
    };*/

    let mut args = Cli::from_args().parse().unwrap();

    // Determine temp directory
    if args.temp_dir.is_none() {
        let mut dir = std::env::temp_dir();
        dir.push("chrono-photo");
        args.temp_dir = Some(dir);
    }
    let temp_dir = args.temp_dir.as_ref().unwrap();
    println!("Temp directory: {:?}", temp_dir);

    // Create temp dir (only 1 level of creation depth)
    if !temp_dir.is_dir() {
        std::fs::create_dir(temp_dir)
            .expect(&format!("Unable to create temp directory {:?}", temp_dir));
        println!("  ... created.");
    }

    println!("{:#?}", args);

    // Convert to time slices and save to temp files
    let (temp_files, layout, size_hint) =
        match to_time_slices(&args.pattern, &args.temp_dir.unwrap()) {
            Ok(fls) => fls,
            Err(err) => {
                println!("{:?}", err.to_string());
                return;
            }
        };

    // Process time slices
    let mut processor = ChronoProcessor::new(args.mode);
    let buff = processor
        .process(&layout, &temp_files[..], Some(size_hint))
        .unwrap();

    image::save_buffer(
        &args.output,
        &buff,
        layout.width,
        layout.height,
        if layout.width_stride == 4 {
            image::ColorType::Rgba8
        } else {
            image::ColorType::Rgb8
        },
    )
    .expect(&format!("Unable to save output file {:?}", &args.output));

    // Delete temp file
    for file in &temp_files {
        match std::fs::remove_file(file) {
            Ok(()) => {}
            Err(err) => println!("Unable to delete file {:?}: {}", file, err.to_string()),
        }
    }
}

fn to_time_slices(
    image_pattern: &str,
    temp_path: &PathBuf,
) -> Result<(Vec<PathBuf>, SampleLayout, usize), TimeSliceError> {
    let images = ImageStream::from_pattern(image_pattern).expect("Error processing pattern");
    TimeSlicer::write_time_slices(images, temp_path.clone())
}
