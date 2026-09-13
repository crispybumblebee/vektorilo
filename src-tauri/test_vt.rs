use std::fs;
use vtracer::{Config, convert_image_to_svg};

fn main() {
    let mut config = Config::default();
    config.color_mode = vtracer::ColorMode::Binary;
    
    // Create a 10x10 white square with a black center 2x2.
    let img = image::RgbaImage::from_fn(10, 10, |x, y| {
        if x > 3 && x < 6 && y > 3 && y < 6 {
            image::Rgba([0, 0, 0, 255])
        } else {
            image::Rgba([255, 255, 255, 255])
        }
    });
    img.save("test.png").unwrap();
    
    convert_image_to_svg("test.png", "test.svg", config).unwrap();
    println!("{}", fs::read_to_string("test.svg").unwrap());
}
