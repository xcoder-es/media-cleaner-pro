use std::path::Path;

use mc_core::{DomainError, ImageDecoder, ImageInfo};

pub struct ImageRsDecoder;

impl ImageRsDecoder {
    pub fn new() -> Self {
        ImageRsDecoder
    }
}

impl ImageDecoder for ImageRsDecoder {
    fn decode(&self, data: &[u8]) -> Result<ImageInfo, DomainError> {
        let reader = image::load_from_memory(data)
            .map_err(|e| DomainError::OperationFailed(format!("decode image: {}", e)))?;

        let width = reader.width();
        let height = reader.height();
        let format = "png";

        Ok(ImageInfo {
            width,
            height,
            format: format.to_string(),
        })
    }
}

pub fn decode_image_from_file(path: &Path) -> anyhow::Result<(u32, u32, String)> {
    let img = image::ImageReader::open(path)?
        .decode()
        .map_err(|e| anyhow::anyhow!("decode error: {}", e))?;

    let width = img.width();
    let height = img.height();
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok((width, height, format))
}
