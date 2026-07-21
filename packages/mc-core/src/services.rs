use crate::domain::*;

pub struct StageProcessor;

impl StageProcessor {
    pub fn exact_duplicate(
        meta: &ImageMetadata,
        seen_hashes: &std::collections::HashSet<String>,
    ) -> StageResult {
        let is_duplicate = seen_hashes.contains(&meta.sha256);
        StageResult {
            stage_name: "Exact Duplicate Removal".to_string(),
            passed: !is_duplicate,
            destination: if is_duplicate {
                Some("duplicates/exact".to_string())
            } else {
                None
            },
            reason: if is_duplicate {
                Some(format!("SHA-256 match: {}", &meta.sha256[..16]))
            } else {
                None
            },
            score: None,
            category: None,
        }
    }

    pub fn perceptual_duplicate(_meta: &ImageMetadata, duplicate_paths: &[String]) -> StageResult {
        let is_duplicate = !duplicate_paths.is_empty();
        StageResult {
            stage_name: "Perceptual Duplicate Removal".to_string(),
            passed: !is_duplicate,
            destination: if is_duplicate {
                Some("duplicates/perceptual".to_string())
            } else {
                None
            },
            reason: if is_duplicate {
                Some(format!(
                    "dHash match with {} file(s)",
                    duplicate_paths.len()
                ))
            } else {
                None
            },
            score: None,
            category: None,
        }
    }

    pub fn tiny_image(meta: &ImageMetadata, min_width: u32, min_height: u32) -> StageResult {
        let is_tiny = meta.width < min_width || meta.height < min_height;
        StageResult {
            stage_name: "Tiny Image Detection".to_string(),
            passed: !is_tiny,
            destination: if is_tiny {
                Some("rejected/tiny".to_string())
            } else {
                None
            },
            reason: if is_tiny {
                Some(format!(
                    "{}x{} below threshold {}x{}",
                    meta.width, meta.height, min_width, min_height
                ))
            } else {
                None
            },
            score: None,
            category: None,
        }
    }

    pub fn icon_detection(meta: &ImageMetadata) -> StageResult {
        let aspect = meta.width as f32 / meta.height.max(1) as f32;
        let is_square = (0.9..=1.1).contains(&aspect);
        let is_small = meta.width <= 256 && meta.height <= 256;
        let is_icon = is_square && is_small;

        StageResult {
            stage_name: "Icon Detection".to_string(),
            passed: !is_icon,
            destination: if is_icon {
                Some("categories/icons".to_string())
            } else {
                None
            },
            reason: if is_icon {
                Some(format!("Square icon {}x{}", meta.width, meta.height))
            } else {
                None
            },
            score: if is_icon { Some(0.95) } else { Some(0.1) },
            category: if is_icon {
                Some("icon".to_string())
            } else {
                None
            },
        }
    }

    pub fn thumbnail_detection(meta: &ImageMetadata) -> StageResult {
        let name_lower = meta.filename.to_lowercase();
        let thumb_patterns = ["thumb", "thumbnail", "small", "mini", "preview", "tn_"];
        let has_thumb_name = thumb_patterns.iter().any(|p| name_lower.contains(p));
        let is_small = meta.width <= 320 || meta.height <= 240;
        let is_thumbnail = has_thumb_name || (is_small && meta.width < meta.height * 2);

        StageResult {
            stage_name: "Thumbnail Detection".to_string(),
            passed: !is_thumbnail,
            destination: if is_thumbnail {
                Some("categories/thumbnails".to_string())
            } else {
                None
            },
            reason: if is_thumbnail {
                Some("Thumbnail pattern detected".to_string())
            } else {
                None
            },
            score: if is_thumbnail { Some(0.9) } else { Some(0.1) },
            category: if is_thumbnail {
                Some("thumbnail".to_string())
            } else {
                None
            },
        }
    }

    pub fn screenshot_detection(meta: &ImageMetadata) -> StageResult {
        let common_resolutions = [
            (1920, 1080),
            (1366, 768),
            (1440, 900),
            (1536, 864),
            (1280, 720),
            (1600, 900),
            (2560, 1440),
            (3840, 2160),
            (1280, 1024),
            (1024, 768),
            (1680, 1050),
            (1920, 1200),
            (1440, 960),
            (2560, 1600),
            (2880, 1800),
            (3024, 1964),
        ];

        let is_common_res = common_resolutions.iter().any(|(w, h)| {
            (meta.width == *w && meta.height == *h) || (meta.width == *h && meta.height == *w)
        });

        let name_lower = meta.filename.to_lowercase();
        let has_screenshot_name = name_lower.contains("screenshot")
            || name_lower.contains("screen shot")
            || name_lower.contains("screencapture");

        let is_screenshot = is_common_res || has_screenshot_name;

        StageResult {
            stage_name: "Screenshot Detection".to_string(),
            passed: !is_screenshot,
            destination: if is_screenshot {
                Some("categories/screenshots".to_string())
            } else {
                None
            },
            reason: if is_screenshot {
                Some(format!("Screenshot: {}x{}", meta.width, meta.height))
            } else {
                None
            },
            score: if is_screenshot {
                Some(0.92)
            } else {
                Some(0.05)
            },
            category: if is_screenshot {
                Some("screenshot".to_string())
            } else {
                None
            },
        }
    }

    pub fn wallpaper_detection(meta: &ImageMetadata) -> StageResult {
        let aspect = meta.width as f32 / meta.height.max(1) as f32;
        let is_ultrawide = (1.8..=3.5).contains(&aspect);
        let is_4k = meta.width >= 3840 || meta.height >= 2160;
        let is_wallpaper = is_ultrawide || (is_4k && aspect >= 1.5);

        StageResult {
            stage_name: "Wallpaper Detection".to_string(),
            passed: !is_wallpaper,
            destination: if is_wallpaper {
                Some("categories/wallpapers".to_string())
            } else {
                None
            },
            reason: if is_wallpaper {
                Some(format!(
                    "Wallpaper aspect {}x{} (ratio {:.2})",
                    meta.width, meta.height, aspect
                ))
            } else {
                None
            },
            score: if is_wallpaper { Some(0.88) } else { Some(0.05) },
            category: if is_wallpaper {
                Some("wallpaper".to_string())
            } else {
                None
            },
        }
    }

    pub fn document_detection(meta: &ImageMetadata) -> StageResult {
        let aspect = meta.width as f32 / meta.height.max(1) as f32;
        let paper_ratios = [1.414, 1.294, 1.545, 1.0];
        let is_paper_ratio = paper_ratios
            .iter()
            .any(|r| (aspect - *r).abs() < 0.15 || ((1.0 / aspect) - *r).abs() < 0.15);
        let is_high_res = meta.width >= 1200;
        let is_document = is_paper_ratio && is_high_res;

        StageResult {
            stage_name: "Document Detection".to_string(),
            passed: !is_document,
            destination: if is_document {
                Some("categories/documents".to_string())
            } else {
                None
            },
            reason: if is_document {
                Some(format!(
                    "Document ratio {:.2} at {}x{}",
                    aspect, meta.width, meta.height
                ))
            } else {
                None
            },
            score: if is_document { Some(0.85) } else { Some(0.1) },
            category: if is_document {
                Some("document".to_string())
            } else {
                None
            },
        }
    }

    pub fn ai_classification(meta: &ImageMetadata) -> StageResult {
        let categories = Self::classify_image(meta);
        let primary = categories
            .first()
            .cloned()
            .unwrap_or_else(|| "uncategorized".to_string());

        StageResult {
            stage_name: "AI Classification".to_string(),
            passed: true,
            destination: Some(format!("categories/{}", primary)),
            reason: Some(format!("Classified as: {}", categories.join(", "))),
            score: Some(0.75),
            category: Some(primary),
        }
    }

    fn classify_image(meta: &ImageMetadata) -> Vec<String> {
        let mut scores: Vec<(&str, f32)> = Vec::new();
        let aspect = meta.width as f32 / meta.height.max(1) as f32;
        let mp = (meta.width * meta.height) as f32 / 1_000_000.0;

        if aspect < 0.8 {
            scores.push(("portrait", 0.7));
        }
        if aspect > 1.3 && mp > 2.0 {
            scores.push(("landscape", 0.6));
        }
        if mp > 5.0 && aspect > 1.0 && aspect < 1.5 {
            scores.push(("macro", 0.5));
        }
        if mp < 0.5 {
            scores.push(("low_quality", 0.8));
        }
        if aspect > 2.5 {
            scores.push(("panorama", 0.9));
        }
        if aspect > 0.95 && aspect < 1.05 {
            scores.push(("social", 0.6));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.into_iter().map(|(cat, _)| cat.to_string()).collect()
    }

    pub fn quality_ranking(meta: &ImageMetadata) -> StageResult {
        let mp = (meta.width * meta.height) as f32 / 1_000_000.0;
        let size_score = (meta.size_bytes as f32 / (mp * 500_000.0)).min(1.0) * 30.0;
        let res_score = mp.min(20.0) / 20.0 * 40.0;
        let aspect_score = if meta.width > meta.height { 15.0 } else { 10.0 };
        let total = size_score + res_score + aspect_score + 15.0;
        let score = total.min(100.0);

        StageResult {
            stage_name: "Quality Ranking".to_string(),
            passed: true,
            destination: Some(format!("quality/{}", Self::quality_bucket(score))),
            reason: Some(format!("Quality score: {:.1}/100", score)),
            score: Some(score as f64),
            category: Some(Self::quality_bucket(score)),
        }
    }

    fn quality_bucket(score: f32) -> String {
        match score {
            s if s >= 90.0 => "excellent".to_string(),
            s if s >= 75.0 => "good".to_string(),
            s if s >= 50.0 => "average".to_string(),
            s if s >= 25.0 => "below_average".to_string(),
            _ => "poor".to_string(),
        }
    }
}

pub struct DuplicateDetector {
    threshold: u32,
    buckets: std::collections::HashMap<u16, Vec<(String, u64)>>,
}

impl DuplicateDetector {
    pub fn new(threshold: u32) -> Self {
        DuplicateDetector {
            threshold,
            buckets: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, path: String, dhash: u64) -> Vec<String> {
        let prefix = (dhash >> 48) as u16;
        let candidates = self.get_candidates(prefix);

        let mut duplicates = Vec::new();
        for (existing_path, existing_hash) in &candidates {
            let distance = hamming_distance(dhash, *existing_hash);
            if distance <= self.threshold {
                duplicates.push(existing_path.clone());
            }
        }

        self.buckets.entry(prefix).or_default().push((path, dhash));
        duplicates
    }

    fn get_candidates(&self, prefix: u16) -> Vec<(String, u64)> {
        let mut candidates = Vec::new();
        for bucket_key in [prefix.wrapping_sub(1), prefix, prefix.wrapping_add(1)] {
            if let Some(bucket) = self.buckets.get(&bucket_key) {
                candidates.extend(bucket.iter().cloned());
            }
        }
        candidates
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }
}

pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}
