use image::{io::Reader as ImageReader, GenericImage, GenericImageView, DynamicImage};
use itertools::Itertools;
use std::{fs::{File, self}, io::Write};

const THRESHOLD: u8 = 60;

fn rle<I: Iterator<Item = bool>>(generator: I) -> Vec<u8> {
    let mut out = Vec::new();

    let mut load = |value: bool, count: u16| {
        // | Marking Bit = 0 | Count - 6 bits | Value bit |
        // | Marking bit = 1 | Count - 14 bits | Value bit |

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

    let mut current = false;
    let mut count: u16 = 0;
    
    for item in generator {
        if count == 0 {
            current = item;
            count += 1;
        } else if item == current {
            count += 1;
            if count == (u16::MAX >> 2) {
                load(current, count);
                count = 0;
            }
        } else {
            load(current, count);

            current = item;
            count = 1;
        }
    }

    if count != 0 {
        load(current, count);
    }

    out
}

fn pixels_column_row(image: &DynamicImage) -> Vec<bool> {
    (0..image.width())
        .cartesian_product(0..image.height())
        .map(|(x, y)| image.get_pixel(x, y))
        .map(|p| p.0[0] > THRESHOLD)
        .collect()
}

fn compress(image: &[bool], _: &[bool]) -> Vec<u8> {
    rle(image.iter().copied())
}

fn compress_delta(current: &[bool], next: &[bool]) -> Vec<u8> {
    // 0 -> stay the same, 1 -> flip bit
    let deltas = current.iter().zip(next.iter())
        .map(|(v1, v2)| v1 != v2);

    rle(deltas)
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
            let value = ( p2 & 0x1 ) * 255;
            let count = ((p as u16 & 0b0111_1111) << 7) | ((p2 as u16) >> 1);
            (value, count)
        }
        else {
            let value = ( p & 0x1 ) * 255;
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

fn apply(algorithm: impl Fn(&[bool], &[bool]) -> Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    for path in fs::read_dir("./images")? {
        let path = path.unwrap();

        paths.push(path)
    }

    paths.sort_by_key(|x| {
        x.file_name()
    });

    let mut images = paths.into_iter()
        .map(|p| p.path())
        .map(|p| ImageReader::open(p).unwrap().decode().unwrap())
        .peekable();

    let mut v = Vec::new();
    while let Some(image) = images.next() {
        let Some(next) = images.peek() else { break };

        let image = pixels_column_row(&image);
        let next = pixels_column_row(next);

        let mut compressed = algorithm(&image, &next);

        // let decompressed = decompress(&compressed).unwrap();
        // let mut f = File::create(format!("images_out/{image_idx:04}.png")).unwrap();
        // decompressed.write_to(&mut f, image::ImageFormat::Png).unwrap();
        // drop(f);

            v.append(&mut compressed);
    }


    Ok(v)
}

fn write(compressed: Vec<u8>) {
    for entry in fs::read_dir("out").unwrap() {
        let Ok(entry) = entry else { continue };
        fs::remove_file(entry.path()).unwrap();
    }

    let mut idx = 0;
    while idx*65000 < compressed.len() {
        let name = format!("out/{idx:02}.bin");
        eprintln!("creating {name}");
        let mut file = File::create(name).unwrap();
        file.write_all(&compressed[idx*65000..((idx+1)*65000).min(compressed.len())]).unwrap();

        idx += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let normal = apply(compress)?;
    let delta = apply(compress_delta)?;

    println!("Normal compression: {} bytes", normal.len());
    println!("Delta compression: {} bytes", delta.len());

    write(delta);

    Ok(())
}
