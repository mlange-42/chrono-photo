use chrono_photo::cli::{Cli, CliParsed};
use chrono_photo::img_stream::ImageStream;
use chrono_photo::time_slice::{TimeSliceError, TimeSlicer};
use image::flat::SampleLayout;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let mut args = CliParsed {
        pattern: "test_data/*.png".to_string(),
        temp_dir: Some(PathBuf::from("test_data/temp")),
    };

    let mut args = Cli::from_args().parse().unwrap();

    // Determine tem directory
    if args.temp_dir.is_none() {
        let mut dir = std::env::temp_dir();
        dir.push("chrono-photo");
        args.temp_dir = Some(dir);
    }
    let temp_dir = args.temp_dir.as_ref().unwrap();
    println!("Temp directory: {:?}", temp_dir);
    if !temp_dir.is_dir() {
        std::fs::create_dir(temp_dir)
            .expect(&format!("Unable to create temp directory {:?}", temp_dir));
        println!("  ... created.");
    }

    // Convert to time slices
    let (temp_files, layout) = match to_time_slices(&args.pattern, &args.temp_dir.unwrap()) {
        Ok(fl) => fl,
        Err(err) => {
            println!("{:?}", err.to_string());
            return;
        }
    };

    // Process time slices

    println!(
        "Created {:?} temp files. Layout: {:?}",
        temp_files.len(),
        layout
    );
}

fn to_time_slices(
    image_pattern: &str,
    temp_path: &PathBuf,
) -> Result<(Vec<PathBuf>, SampleLayout), TimeSliceError> {
    let images = ImageStream::from_pattern(image_pattern).expect("Error processing pattern");

    TimeSlicer::write_time_slices(images, temp_path.clone())
}
