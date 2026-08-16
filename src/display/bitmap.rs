use std::path::Path;

use image::{imageops::rotate90, DynamicImage, GrayImage};
use thiserror::Error;

use super::BITMAP_SIZE;

#[derive(Debug, Error)]
pub enum BitmapError {
    #[error("failed to read or write display image")]
    Image(#[from] image::ImageError),
    #[error("rendered bitmap has {actual} bytes; expected {expected}")]
    InvalidSize { actual: usize, expected: usize },
}

fn image_to_monochrome(image: DynamicImage) -> GrayImage {
    let mut image = image.grayscale().to_luma8();

    for pixel in image.iter_mut() {
        *pixel = 255 - *pixel;
    }

    image
}

fn pack_bitmap(image: &GrayImage) -> Vec<u8> {
    image
        .chunks(8)
        .map(|pixels| {
            pixels
                .iter()
                .enumerate()
                .fold(0_u8, |byte, (index, pixel)| {
                    if *pixel > 128 {
                        byte | (1 << (7 - index))
                    } else {
                        byte
                    }
                })
        })
        .collect()
}

pub fn build_bitmap(
    source_image_path: &Path,
    converted_image_path: &Path,
) -> Result<Vec<u8>, BitmapError> {
    let source = image::open(source_image_path)?;
    let converted = rotate90(&image_to_monochrome(source));
    converted.save(converted_image_path)?;

    let bitmap = pack_bitmap(&converted);
    if bitmap.len() != BITMAP_SIZE {
        return Err(BitmapError::InvalidSize {
            actual: bitmap.len(),
            expected: BITMAP_SIZE,
        });
    }

    Ok(bitmap)
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, Luma};

    use super::pack_bitmap;
    use crate::display::{BITMAP_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH};

    #[test]
    fn packs_one_byte_from_eight_pixels() {
        let image = GrayImage::from_raw(8, 1, vec![255, 0, 255, 0, 255, 0, 255, 0]).unwrap();

        assert_eq!(pack_bitmap(&image), vec![0b1010_1010]);
    }

    #[test]
    fn packs_exactly_one_full_display() {
        let image = GrayImage::from_pixel(DISPLAY_WIDTH, DISPLAY_HEIGHT, Luma([255]));

        assert_eq!(pack_bitmap(&image).len(), BITMAP_SIZE);
    }
}
