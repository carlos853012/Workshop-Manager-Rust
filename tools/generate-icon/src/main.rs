use std::fs::File;
use std::io::{BufWriter, Write};
use workshop_common::icon_data::generate_wrench_icon;

fn main() {
    let sizes = [16u32, 32, 48, 64, 128, 256];
    let bg = [37u8, 99, 235]; // #2363eb
    let fg = [255u8, 255, 255]; // white

    // Generate PNG data for each size
    let mut images: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in &sizes {
        let pixels = generate_wrench_icon(size, bg, fg);
        let png = encode_png(size, &pixels);
        images.push((size, png));
    }

    // Write .ico file
    let mut ico = Vec::new();

    // ICONDIR header
    ico.extend_from_slice(&[0, 0]); // reserved
    ico.extend_from_slice(&[1, 0]); // type: icon
    ico.extend_from_slice(&(images.len() as u16).to_le_bytes());

    // Calculate data offset
    let header_size = 6 + (images.len() as u32) * 16;
    let mut data_offset = header_size;

    // ICONDIRENTRY for each image
    for (size, png) in &images {
        let width = if *size >= 256 { 0 } else { *size as u8 };
        let height = if *size >= 256 { 0 } else { *size as u8 };

        ico.push(width);
        ico.push(height);
        ico.push(0); // color count
        ico.push(0); // reserved
        ico.extend_from_slice(&[1, 0]); // color planes
        ico.extend_from_slice(&[32, 0]); // bits per pixel
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&data_offset.to_le_bytes());

        data_offset += png.len() as u32;
    }

    // PNG data
    for (_, png) in &images {
        ico.extend_from_slice(png);
    }

    // Write to file
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join("workshop-viewer")
        .join("icon.ico");
    let file = File::create(&path).expect("Failed to create icon.ico");
    let mut writer = BufWriter::new(file);
    writer.write_all(&ico).expect("Failed to write icon.ico");

    println!("Generated icon.ico at {:?}", path);
    println!("Sizes: {:?}", sizes);
}

fn encode_png(width: u32, rgba: &[u8]) -> Vec<u8> {
    let height = width; // square icon
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(rgba).unwrap();
        writer.finish().unwrap();
    }
    buf
}
