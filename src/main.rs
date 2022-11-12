use byteorder::WriteBytesExt;
use image::{io::Reader as ImageReader, GenericImage, GenericImageView};
use std::{fs::{File, self}, io::Write, str::FromStr, path::Path};

fn compress(path: &std::path::Path) -> image::ImageResult<Vec<u8>> {
    let img = ImageReader::open(path)?.decode()?;

    /*
    | Marking Bit | Count - 6 bits | Value bit |
    OR
    | Marking bit | Count - 14 bits | Value bit |
    */

    let load_into = |out: &mut Vec<u8>, value: bool, count: u16| {
        // If count fits in 6 bits
        if count <= (u8::MAX >> 2).into() {
            // ( Marking bit is false)
            out.push(((count as u8) << 1) | (value as u8));
        }
        else {
            out.push((0b1000_0000) | ((count >> 7) as u8)); // Get 7 top bits of count and activate marking bit
            out.push((((count as u8) & 0b0111_1111) << 1) | (value as u8)); // Get 7 bottom bits of count and pack value in
        }
    };

    let mut out = Vec::new();

    let mut value: bool = false;
    let mut count: u16 = 0;

    for x in 0..img.width() {
        for y in 0..img.height() {
            let p = img.get_pixel(x, y).0[0];
            if count == 0 {
                value = p > 60;
                count += 1;
            }
            else {
                let current_value = p > 60;

                if value == current_value {
                    count += 1;
                    if count == (u16::MAX >> 2) {
                        load_into(&mut out, value, count);
                        count = 0;
                    }
                }
                else {
                    load_into(&mut out, value, count);

                    value = current_value;
                    count = 1;
                }
            }
        }
    }

    if count != 0 {
        load_into(&mut out, value, count);
    }

    Ok(out)
}

fn decompress(compressed: &[u8]) -> image::ImageResult<image::DynamicImage> {
    const WIDTH: u32 = 160;
    const HEIGHT: u32 = 120;

    let mut img = image::DynamicImage::new_rgb8(WIDTH, HEIGHT);

    let mut x = 0;
    let mut y = 0;

    let mut it = compressed.iter();
    while let Some(&p) = it.next() {
        let (value, count): (u8, u16) = if (p & 0b1000_0000) > 0 { // if you've got the marking bit, you need to consoom the next package too
            let p2 = *it.next().unwrap();
            let value = ( p2 & 0x1 ) as u8 * 255;
            let count = ((p as u16 & 0b0111_1111) << 7) | ((p2 as u16) >> 1);
            (value, count)
        }
        else {
            let value = ( p & 0x1 ) as u8 * 255;
            let count = ( p & (!0x1) ) >> 1;
            (value, count.into())
        };

        for _ in 0..count {
            img.put_pixel(x, y, image::Rgba( [ value, value, value, 255 ] ));

            y += 1;
            if y == HEIGHT {
                y = 0;
                x += 1;
            }
        }
    }

    Ok(img)
}

fn byte_array_to_c(compressed: &[u8]) -> String {
    let mut s = "\n".to_string();

    let mut idx = 0;
    for &b in compressed {
        s.push_str(format!("{:#04x},", b).as_str());

        idx += 1;
        if idx == 16 {
            s.push('\n');
            idx = 0;
        }
    }
    s.push('\n');

    s
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    for path in fs::read_dir("./images")? {
        let path = path.unwrap();

        paths.push(path)
    }

    paths.sort_by(|a, b| {
        let numa = a.file_name().into_string().unwrap().chars().filter(|x| x.is_numeric()).collect::<String>().parse::<u32>().unwrap();
        let numb = b.file_name().into_string().unwrap().chars().filter(|x| x.is_numeric()).collect::<String>().parse::<u32>().unwrap();
        numa.cmp(&numb)
    });

    // let mut s = "const uint8_t frames[] = {".to_string();
    // let mut s_num = "const uint24_t FRAME_NUMS[] = {".to_string();

    let mut v = Vec::with_capacity(65000);
    let mut idx = 0;
    for path in paths {
        println!("{:?}", path);

        let mut compressed = compress(&path.path())?;

        if v.len() + compressed.len() < 65000 {
            v.append(&mut compressed);
        }
        else {
            let mut file = File::create(format!("out/{}.bin", idx))?;
            file.write_all(&v)?;

            v.clear();
            v.append(&mut compressed);

            idx += 1;
        }

        // s.push_str(byte_array_to_c(&compressed).as_str());
        // s_num.push_str(format!("{}, ", compressed.len()).as_str());

        // let mut file = File::create(format!("outi/{}", path.file_name().to_str().unwrap()))?;
        // let uncompressed = decompress(&compressed)?;
        // uncompressed.write_to(&mut file, image::ImageFormat::Png)?;
    }

    // s.push_str("\n};\n");
    // s_num.push_str("};\n");

    // let mut file = File::create("out_text.h")?;
    // file.write_all(b"const uint24_t FRAME_NUM = 100;\n")?;
    // file.write_all(s_num.as_bytes())?;
    // file.write_all(s.as_bytes())?;

    Ok(())
}
