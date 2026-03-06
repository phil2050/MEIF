use macroquad::prelude::*;
use colored::Colorize;
use std::io::{ Read, Write };

mod utils;

#[macroquad::main("MEIF Viewer")]
async fn main() {
    let passed = "==>".blue();
    let failed = "! >".red();
    let mut args = std::env::args();
    let _program = args.next();
    let filename = args.next().expect("Please provide an MEIF (Minimally Efficient Image Format) file as an argument.");
    let mut export_path: Option<String> = None;
    while let Some(arg) = args.next() {
        if arg == "--export" || arg == "-o" {
            if let Some(path) = args.next() {
                export_path = Some(path);
            } else {
                eprintln!("Missing path after {}", arg);
                std::process::exit(1);
            }
        }
    }
    let mut file = std::fs::File::open(&filename).expect("Failed to open the file. Does it exist?");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read the file.");
    println!("{} Read {} bytes from file {}", passed, utils::format_bytes(buffer.len()).to_string().blue(), filename.green());
    let mut parser = utils::MEIFParser::new(buffer);
    if parser.non_advancing_next_bytes(b"MEIF") {
        match parser.parse() {
            Ok(image) => {
                println!("{} Parsed MEIF file successfully.", passed);
                println!("{} {:?}", passed, image);
                if let Some(path) = export_path {
                    let rgb = image.to_rgb_bytes();
                    let out_path = std::path::Path::new(&path);
                    if let Err(e) = image::save_buffer_with_format(
                        out_path,
                        &rgb,
                        image.width,
                        image.height,
                        image::ColorType::Rgb8,
                        image::ImageFormat::Jpeg,
                    ) {
                        eprintln!("{} Failed to export JPEG: {}", failed, e);
                        std::process::exit(1);
                    }
                    println!("{} Exported JPEG to {}", passed, out_path.display().to_string().green());
                    return;
                }
                for index in &image.indexes {
                    print!("{}", "██".truecolor(
                        (index.r * 255.0).round() as u8,
                        (index.g * 255.0).round() as u8,
                        (index.b * 255.0).round() as u8
                    ));
                }
                println!();
                // Show the image
                let mut bytes = Vec::new();
                for i in 0..image.data.len() {
                    let data = image.data[i];
                    bytes.push((image.indexes[data as usize].r * 255.0).round() as u8);
                    bytes.push((image.indexes[data as usize].g * 255.0).round() as u8);
                    bytes.push((image.indexes[data as usize].b * 255.0).round() as u8);
                    bytes.push((image.indexes[data as usize].a * 255.0).round() as u8);
                }
                let texture = Texture2D::from_rgba8(image.width as u16, image.height as u16, &bytes);
                texture.set_filter(FilterMode::Nearest);
                loop {
                    let sw = screen_width();
                    let sh = screen_height();
                    let image_width = image.width as f32;
                    let image_height = image.height as f32;
                    let display_image_size = if image_width / image_height > sw / sh {
                        Vec2::new(sw, sw * (image_height / image_width))
                    } else {
                        Vec2::new(sh * (image_width / image_height), sh)
                    };

                    clear_background(BLACK);
                    draw_texture_ex(
                        &texture,
                        (sw - display_image_size.x) / 2.0,
                        (sh - display_image_size.y) / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(display_image_size),
                            ..Default::default()
                        },
                    );
                    next_frame().await;
                }
            }
            Err(e) => println!("{} Error parsing MEIF file: {}", failed, e.message),
        }
    } else {
        println!("Not a MEIF file, trying to convert from \"\"\"normal\"\"\" (whatever that means) image format.");
        let image = image::open(&filename).expect("Failed to open the image file.");
        let meif_converter = utils::MEIFConverter::new(image);
        match meif_converter.convert() {
            Ok(meif_image) => {
                // Show the actual error by the parser
                match utils::MEIFParser::new(meif_image.to_bytes()).parse() {
                    Ok(_) => println!("{} Converted image to MEIF format successfully.", passed),
                    Err(e) => {
                        println!("{} Error converting image to MEIF format: {}", failed, e.message);
                        std::process::exit(1);
                    }
                }
                let output_filename = format!("{}.meif", filename);
                let mut output_file = std::fs::File::create(&output_filename).expect("Failed to create output file.");
                output_file.write_all(&meif_image.to_bytes()).expect("Failed to write to output file.");
                println!("{} Wrote {} bytes to file {}", passed, utils::format_bytes(meif_image.to_bytes().len()).to_string().blue(), output_filename.green());
                println!("{} Saved MEIF image to {}", passed, output_filename.green());
                let name = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_name().map(|f| f.to_os_string()))
                    .and_then(|s| s.into_string().ok())
                    .unwrap_or_else(|| "<unknown>".to_string());
                println!("{} Now run: `{} {} {}`", passed, "$".yellow(), name.yellow(), output_filename.green());
            }
            Err(e) => println!("{} Error converting image to MEIF format: {}", failed, e.message),
        }
    }
}
