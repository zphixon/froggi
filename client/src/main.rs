//use image::io::Reader;
//use image::GenericImageView;
//
//use froggi::request::RequestKind;
//
//use std::io::Cursor;

use image::ImageEncoder;

fn main() {
    let size = 50.0;

    let bytes = include_bytes!("C:\\Windows\\Fonts\\NotoSans-Regular.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
    let fonts = &[font];

    let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp);
    layout.append(fonts, &fontdue::layout::TextStyle::new("bingo ", size, 0));
    layout.append(fonts, &fontdue::layout::TextStyle::new("bongo", size, 0));

    let mut metrics = Vec::new();
    let mut bitmaps = Vec::new();

    layout.glyphs().iter().for_each(|glyph| {
        let (metric, bitmap) = fonts[0].rasterize_config(glyph.key);
        metrics.push(metric);
        bitmaps.push(bitmap);
    });

    let width = metrics.iter().map(|m| m.advance_width).sum::<f32>().ceil() as u32;
    let height = metrics
        .iter()
        .map(|m| m.height + m.ymin.abs() as usize)
        .max()
        .unwrap() as u32;
    dbg!(width, height);

    let mut image = image::ImageBuffer::new(width, height);
    for i in 0..image.width() {
        for j in 0..image.height() {
            image.put_pixel(i, j, image::Rgba([255u8, 255, 255, 255]));
        }
    }

    let mut x: f32 = 0.;
    for ((metric, bitmap), glyph) in metrics.iter().zip(bitmaps.iter()).zip(layout.glyphs()) {
        println!("draw {} at {:?} {}", glyph.key.c, metric, bitmap.len());
        //let mut row = 0;
        if metric.width > 0 {
            for (y, row) in bitmap.chunks(metric.width).enumerate() {
                for (col, coverage) in row.iter().enumerate() {
                    let value = 255 - coverage.clamp(&1, &255);
                    let x = x.floor() as u32 + col as u32;
                    let y = (y as i32 + metric.ymin.abs()) as u32;
                    if glyph.key.c == 'n' {
                        println!("{} {}", x, y);
                    }
                    image.put_pixel(x, y, image::Rgba([value, value, value, 255]))
                }
            }
        }
        //for (i, pixel) in bitmap.iter().enumerate() {
        //    if i + 1 == metric.width {
        //        row += 1;
        //    }
        //    let pixel_x = metric.xmin as u32 + (i % metric.width) as u32;
        //    let pixel_x = pixel_x + x.floor() as u32;
        //    let pixel_y = metric.ymin + row;
        //    let pixel_y = pixel_y.abs() as u32;

        //    println!("pp {} {} {}", row, pixel_x, pixel_y);

        //    image.put_pixel(pixel_x, pixel_y, image::Rgba([255u8, 255u8, 255u8, 255u8]));
        //}
        x += metric.advance_width;
    }

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
