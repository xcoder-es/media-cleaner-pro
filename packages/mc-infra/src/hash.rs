use image::{DynamicImage, GenericImageView};
use sha2::{Digest, Sha256};

use mc_core::{DomainError, ExactHasher, ImageHasher};

pub struct Sha256Hasher;

impl Sha256Hasher {
    pub fn new() -> Self {
        Sha256Hasher
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ExactHasher for Sha256Hasher {
    fn compute_sha256(&self, data: &[u8]) -> Result<String, DomainError> {
        let hash = Sha256::digest(data);
        Ok(format!("{:x}", hash))
    }
}

pub struct DHashHasher;

impl DHashHasher {
    pub fn new() -> Self {
        DHashHasher
    }
}

impl Default for DHashHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageHasher for DHashHasher {
    fn compute_dhash(&self, data: &[u8]) -> Result<u64, DomainError> {
        let img = image::load_from_memory(data)
            .map_err(|e| DomainError::OperationFailed(format!("decode image for dhash: {}", e)))?;
        Ok(compute_dhash(&img))
    }

    fn hamming_distance(&self, a: u64, b: u64) -> u32 {
        mc_core::hamming_distance(a, b)
    }
}

fn compute_dhash(img: &DynamicImage) -> u64 {
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

pub fn compute_sha256_from_file(path: &std::path::Path) -> anyhow::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn compute_dhash_from_file(path: &std::path::Path) -> anyhow::Result<u64> {
    let img = image::ImageReader::open(path)?
        .decode()
        .map_err(|e| anyhow::anyhow!("decode error: {}", e))?;
    Ok(compute_dhash(&img))
}
