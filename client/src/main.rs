//use image::io::Reader;
//use image::GenericImageView;
//
//use froggi::request::RequestKind;
//
//use std::io::Cursor;

use image::ImageEncoder;

fn main() {
    let size = 50.0;

    let bytes = include_bytes!("C:\\Windows\\Fonts\\cmunrm.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
    let fonts = &[font];

    // you have to use Layout to get more than one glyph
    let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp);
    layout.append(
        fonts,
        &fontdue::layout::TextStyle::new("Judge my vow ^178", size, 0),
    );

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
    let mut image = image::ImageBuffer::new(width, height);
    for i in 0..image.width() {
        for j in 0..image.height() {
            image.put_pixel(i, j, image::Rgba([255u8, 255, 255, 255]));
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
                    image.put_pixel(x, y, image::Rgba([value, value, value, 255]));
                }
            }
        }
        // move along by glyph advance width
        x += metric.advance_width;
    }

    // write the image
    let image_file = std::fs::File::create("image.png").unwrap();
    let image_writer = std::io::BufWriter::new(image_file);
    let image_encoder = image::codecs::png::PngEncoder::new(image_writer);
    image_encoder
        .write_image(&image, width, height, image::ColorType::Rgba8)
        .unwrap();

    //let (metrics, bitmap) = fonts[0].rasterize('a', size);

    //dbg!(font);
    //dbg!(metrics);
    //for (k, v) in bitmap.iter().enumerate() {
    //    print!("{:03} ", v);
    //    if (k + 1) % metrics.width == 0 {
    //        println!();
    //    }
    //}

    //let mut image = Vec::new();
    //bitmap
    //    .iter()
    //    .zip(bitmap.iter())
    //    .zip(bitmap.iter())
    //    .for_each(|((r, g), b)| {
    //        image.push(*r);
    //        image.push(*g);
    //        image.push(*b);
    //        image.push(0xff);
    //    });

    //let image_file = std::fs::File::create("image.png").unwrap();
    //let image_writer = std::io::BufWriter::new(image_file);
    //let image_encoder = image::codecs::png::PngEncoder::new(image_writer);
    //image_encoder
    //    .write_image(
    //        &image,
    //        metrics.width as u32,
    //        metrics.height as u32,
    //        image::ColorType::Rgba8,
    //    )
    //    .unwrap();

    /*let local = std::env::args().collect::<String>().contains("-l");
    //let server = include_str!("../server_address").trim();
    let server = "gang-and-friends.com:11121";

    let addr = if local { "127.0.0.1:11121" } else { server };

    println!(
        "connecting to {}",
        if local { addr } else { "a secret server" }
    );

    let result = froggi::send_request(addr, "test_markup.fml", RequestKind::Page).unwrap();

    println!("got {:#?}", result);
    match result.parse() {
        Ok(page) => {
            println!("page ok: {:#?}", page);
        }

        Err(errors) => {
            for error in errors {
                println!("{}", error);
            }
        }
    }

    if result.items().len() > 0 {
        let reader = Reader::new(Cursor::new(result.items()[0].data()))
            .with_guessed_format()
            .expect("cursor never fails");
        let format = reader.format();
        match reader.decode() {
            Ok(image) => println!("got image: {:?} {:?}", format, image.dimensions()),
            Err(error) => println!("couldn't decode: {}", error),
        }
    }
     */
}
