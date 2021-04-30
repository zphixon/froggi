use image::{codecs::png::PngEncoder, ColorType, EncodableLayout, ImageBuffer, ImageEncoder, Rgba};

use fontdue::{
    layout::{CoordinateSystem, Layout, TextStyle},
    Font, FontSettings,
};

use std::{fs, fs::File, io::BufWriter};

fn main() {
    let size = 50.0;

    let args = std::env::args().collect::<Vec<String>>();

    let text = if args.len() >= 2 {
        args[1].clone()
    } else {
        "Judge my vow ^178".to_string()
    };

    let font_file = if args.len() >= 3 {
        args[2].clone()
    } else {
        "C:\\Windows\\Fonts\\cmunrm.ttf".to_string()
    };

    println!("using font '{}' to render '{}'", font_file, text);

    let bytes = fs::read(font_file).unwrap();
    let font = Font::from_bytes(bytes.as_bytes(), FontSettings::default()).unwrap();
    let fonts = &[font];

    // you have to use Layout to get more than one glyph
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    layout.append(fonts, &TextStyle::new(&text, size, 0));

    // collect the metrics and coverage bitmaps
    let mut metrics = Vec::new();
    let mut bitmaps = Vec::new();

    // rasterize each glyph
    layout.glyphs().iter().for_each(|glyph| {
        // TODO ligatures?
        let (metric, bitmap) = fonts[0].rasterize_config(glyph.key);
        metrics.push(metric);
        bitmaps.push(bitmap);
    });

    // TODO xmin?
    // width of the resulting image
    let width = metrics.iter().map(|m| m.advance_width).sum::<f32>().ceil() as u32;

    // baseline - the bottom of letters like A or the underside of letters like lowercase q
    // in image space (pixels from the top)
    let baseline = metrics
        .iter()
        .map(|m| m.height as i32 + m.ymin)
        .max()
        .unwrap();

    // lowest tail - how far below the baseline the longest dropping letters like g go
    let lowest_tail = metrics.iter().map(|m| m.ymin).min().unwrap();

    // height of the image in pixels
    let height = baseline as u32
        + if lowest_tail.is_negative() {
            lowest_tail.abs() as u32
        } else {
            0
        }
        + 1; // need +1 because we work with pixels instead of floats to make it easier

    // clear the image with white
    let mut image = ImageBuffer::new(width, height);
    for i in 0..image.width() {
        for j in 0..image.height() {
            image.put_pixel(i, j, Rgba([255u8, 255, 255, 255]));
        }
    }

    // the current x coordinate in image space
    let mut x: f32 = 0.;
    for (metric, bitmap) in metrics.iter().zip(bitmaps.iter()) {
        // ignore glyphs with no width like spaces
        if metric.width > 0 {
            // write each row, from bottom to top
            for (row_index, row) in bitmap.chunks(metric.width).rev().enumerate() {
                for (col, coverage) in row.iter().enumerate() {
                    // y pixel in image space is baseline offset by ymin, offset by the row index
                    let y = (baseline as i32 - metric.ymin) as u32 - row_index as u32;
                    // x pixel in image space is how far along we are
                    let x = x as u32 + col as u32;

                    // value is white minus coverage (darker where more coverage)
                    let value = 255 - coverage.clamp(&0, &255);
                    // TODO mix alpha
                    image.put_pixel(x, y, Rgba([value, value, value, 255]));
                }
            }
        }
        // move along by glyph advance width
        x += metric.advance_width;
    }

    // write the image
    let image_file = File::create("image.png").unwrap();
    let image_writer = BufWriter::new(image_file);
    let image_encoder = PngEncoder::new(image_writer);
    image_encoder
        .write_image(&image, width, height, ColorType::Rgba8)
        .unwrap();
}
