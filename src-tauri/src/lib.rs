use image::Rgba;
use std::fs;
use tauri::command;
use vtracer::Config;

fn generate_svg_internal(
    image_bytes: &[u8],
    threshold: u8,
    invert: bool,
    detail_level: u8,
    remove_background: bool
) -> Result<String, String> {
    // Load image and apply threshold
    let img = image::load_from_memory(image_bytes).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();

    for pixel in rgba.pixels_mut() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        
        let v = if a == 0 {
            255 // Transparent is always white (background)
        } else {
            let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
            let mut val = if luminance >= threshold { 255 } else { 0 };
            if invert {
                val = 255 - val;
            }
            val
        };
        
        let alpha = if remove_background && v == 255 { 0 } else { 255 };
        *pixel = Rgba([v, v, v, alpha]);
    }

    // Save thresholded image to temp file
    let temp_dir = std::env::temp_dir();
    let temp_png = temp_dir.join("vektorilo_temp.png");
    let temp_svg = temp_dir.join("vektorilo_temp.svg");

    rgba.save(&temp_png).map_err(|e| e.to_string())?;

    // Run vtracer
    let mut config = Config::default();
    config.color_mode = vtracer::ColorMode::Binary;
    config.hierarchical = vtracer::Hierarchical::Stacked;
    config.color_precision = 6;
    config.layer_difference = 16;
    config.splice_threshold = 45;
    config.path_precision = Some(8);

    match detail_level {
        1 => {
            config.filter_speckle = 10;
            config.corner_threshold = 90;
            config.length_threshold = 8.0;
            config.max_iterations = 10;
        },
        3 => {
            config.filter_speckle = 2;
            config.corner_threshold = 30;
            config.length_threshold = 2.0;
            config.max_iterations = 10;
        },
        _ => {
            config.filter_speckle = 4;
            config.corner_threshold = 60;
            config.length_threshold = 4.0;
            config.max_iterations = 10;
        }
    }

    vtracer::convert_image_to_svg(&temp_png, &temp_svg, config)
        .map_err(|e| e.to_string())?;

    // Read generated SVG
    let svg_content = fs::read_to_string(&temp_svg).map_err(|e| e.to_string())?;

    // The "Cricut Fix"
    let cricut_svg = apply_cricut_fix(&svg_content);

    // Cleanup
    let _ = fs::remove_file(temp_png);
    let _ = fs::remove_file(temp_svg);

    Ok(cricut_svg)
}

#[command]
async fn generate_svg_preview(
    image_bytes: Vec<u8>, 
    threshold: u8, 
    invert: bool,
    detail_level: u8,
    remove_background: bool
) -> Result<String, String> {
    generate_svg_internal(&image_bytes, threshold, invert, detail_level, remove_background)
}

#[command]
async fn process_and_save_svg(
    image_bytes: Vec<u8>, 
    threshold: u8, 
    file_name: String,
    invert: bool,
    detail_level: u8,
    remove_background: bool
) -> Result<(), String> {
    let base_name = std::path::Path::new(&file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let default_name = format!("{}.svg", base_name);

    let save_path = match rfd::FileDialog::new()
        .add_filter("SVG Image", &["svg"])
        .set_file_name(&default_name)
        .save_file()
    {
        Some(p) => p,
        None => return Ok(()),
    };

    let svg_content = generate_svg_internal(&image_bytes, threshold, invert, detail_level, remove_background)?;

    fs::write(save_path, svg_content).map_err(|e| e.to_string())?;

    Ok(())
}

fn apply_cricut_fix(svg: &str) -> String {
    let mut fixed = svg.replace("standalone=\"no\"", "");
    // Strip any paths that were mistakenly filled with white if vtracer outputs them
    fixed = fixed.replace("fill=\"#ffffff\"", "fill=\"none\"");
    fixed = fixed.replace("fill=\"#FFFFFF\"", "fill=\"none\"");
    fixed
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            process_and_save_svg,
            generate_svg_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
