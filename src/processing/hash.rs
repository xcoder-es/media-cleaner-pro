use image::{DynamicImage, GenericImageView};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn compute_sha256(path: &Path) -> anyhow::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute dHash (difference hash) for perceptual similarity
/// Resize to 9x8, grayscale, compare adjacent pixels
pub fn compute_dhash(img: &DynamicImage) -> u64 {
    let gray = img.resize_exact(9, 8, image::imageops::FilterType::Lanczos3);
    let mut hash: u64 = 0;

    for y in 0..8 {
        for x in 0..8 {
            let left = gray.get_pixel(x, y)[0];
            let right = gray.get_pixel(x + 1, y)[0];
            if right > left {
                hash |= 1 << (y * 8 + x);
            }
        }
    }

    hash
}

pub use mc_core::hamming_distance;
pub use mc_core::format_dhash;
