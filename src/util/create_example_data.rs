extern crate image;

use std::path::PathBuf;

fn main() {
    let size = (1024, 768);
    let channels = 4;
    let num_images = 25;
    let radius = 8;
    let path = "test_data/generated";
    let dir = PathBuf::from(path);
    if !dir.is_dir() {
        std::fs::create_dir_all(&dir).expect(&format!("Unable to create directory {:?}", &dir));
        println!("  ... created.");
    }

    let buff_len = size.0 * size.1 * channels;
    let mut buffer = vec![0_u8; buff_len];
    for img in 0..num_images {
        for i in 0..buff_len {
            buffer[i] = 255;
        }
        let (cx, cy) = (100 + img * 10, size.1 / 3 + img * 5);
        for xx in (cx - radius)..=(cx + radius) {
            for yy in (cy - radius)..=(cy + radius) {
                let idx = xy_to_index(size, channels, xx, yy);
                for ch in idx..(idx + 3) {
                    buffer[ch] = 0;
                }
                //println!("{:?}", &buffer[idx..(idx + 3)]);
            }
        }
        let mut out_path = path.to_string();
        out_path.push_str(&format!("/image-{:05}.png", img));
        image::save_buffer(
            &out_path,
            &buffer,
            size.0 as u32,
            size.1 as u32,
            if channels == 4 {
                image::ColorType::Rgba8
            } else {
                image::ColorType::Rgb8
            },
        )
        .unwrap();
    }
}

fn xy_to_index(size: (usize, usize), channels: usize, x: usize, y: usize) -> usize {
    y * size.0 * channels + x * channels
}
/*fn index_to_xy(size: (usize, usize), channels: usize, index: usize) -> (usize, usize) {
    ((index / channels) & size.0, (index / channels) / size.0)
}*/
