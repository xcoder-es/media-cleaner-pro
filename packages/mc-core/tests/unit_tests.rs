use mc_core::*;
use std::collections::HashSet;
use std::path::Path;

#[test]
fn test_job_id_serde_roundtrip() {
    let id = JobId("abc".to_string());
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: JobId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn test_user_id_serde_roundtrip() {
    let id = UserId("user-1".to_string());
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: UserId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn test_team_id_serde_roundtrip() {
    let id = TeamId("team-42".to_string());
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: TeamId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

fn make_meta(
    filename: &str,
    width: u32,
    height: u32,
    size_bytes: u64,
    sha256: &str,
    dhash: Option<u64>,
) -> ImageMetadata {
    ImageMetadata {
        path: format!("/test/{}", filename),
        filename: filename.to_string(),
        size_bytes,
        width,
        height,
        sha256: sha256.to_string(),
        dhash,
        format: filename.rsplit('.').next().unwrap_or("png").to_string(),
    }
}

#[test]
fn test_job_id_display() {
    let id = JobId("abc-123".to_string());
    assert_eq!(id.to_string(), "abc-123");
}

#[test]
fn test_job_id_from_string() {
    let id: JobId = "test-id".to_string().into();
    assert_eq!(id.as_ref(), "test-id");
}

#[test]
fn test_job_id_from_str() {
    let id = JobId::from("hello");
    assert_eq!(id.as_ref(), "hello");
}

#[test]
fn test_job_id_eq_hash() {
    let a = JobId("same".to_string());
    let b = JobId("same".to_string());
    let c = JobId("other".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&JobId("same".to_string())));
    assert!(!set.contains(&JobId("nope".to_string())));
}

#[test]
fn test_user_id_eq_hash() {
    let a = UserId("u1".to_string());
    let b = UserId("u1".to_string());
    let c = UserId("u2".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&UserId("u1".to_string())));
}

#[test]
fn test_team_id_eq_hash() {
    let a = TeamId("t1".to_string());
    let b = TeamId("t1".to_string());
    let c = TeamId("t2".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&TeamId("t1".to_string())));
}

#[test]
fn test_user_id_display() {
    let id = UserId("user-1".to_string());
    assert_eq!(id.to_string(), "user-1");
}

#[test]
fn test_user_id_from_string() {
    let id: UserId = "user-1".to_string().into();
    assert_eq!(id.as_ref(), "user-1");
}

#[test]
fn test_user_id_from_str() {
    let id = UserId::from("user-1");
    assert_eq!(id.as_ref(), "user-1");
}

#[test]
fn test_team_id_display() {
    let id = TeamId("team-42".to_string());
    assert_eq!(id.to_string(), "team-42");
}

#[test]
fn test_team_id_from_string() {
    let id: TeamId = "team-42".to_string().into();
    assert_eq!(id.as_ref(), "team-42");
}

#[test]
fn test_team_id_from_str() {
    let id = TeamId::from("team-42");
    assert_eq!(id.as_ref(), "team-42");
}

#[test]
fn test_is_image_file_valid_extensions() {
    for ext in &["jpg", "jpeg", "png", "bmp", "webp", "gif", "tiff", "tif"] {
        assert!(
            is_image_file(Path::new(&format!("photo.{}", ext))),
            "failed for .{}",
            ext
        );
    }
}

#[test]
fn test_is_image_file_uppercase() {
    assert!(is_image_file(Path::new("photo.JPG")));
    assert!(is_image_file(Path::new("photo.PNG")));
}

#[test]
fn test_is_image_file_invalid() {
    assert!(!is_image_file(Path::new("document.pdf")));
    assert!(!is_image_file(Path::new("video.mp4")));
    assert!(!is_image_file(Path::new("archive.zip")));
    assert!(!is_image_file(Path::new("file")));
}

#[test]
fn test_format_duration_zero() {
    assert_eq!(format_duration(0), "00:00:00");
}

#[test]
fn test_format_duration_typical() {
    assert_eq!(format_duration(3661), "01:01:01");
    assert_eq!(format_duration(86399), "23:59:59");
}

#[test]
fn test_format_dhash_zero() {
    assert_eq!(format_dhash(0), "0000000000000000");
}

#[test]
fn test_format_dhash_value() {
    assert_eq!(format_dhash(0xDEADBEEF), "00000000DEADBEEF");
    assert_eq!(format_dhash(!0), "FFFFFFFFFFFFFFFF");
}

#[test]
fn test_hamming_distance_identical() {
    assert_eq!(hamming_distance(0x1234, 0x1234), 0);
}

#[test]
fn test_hamming_distance_different() {
    assert_eq!(hamming_distance(0x0000, 0xFFFF), 16);
    assert_eq!(hamming_distance(0x0000, 0x0001), 1);
}

#[test]
fn test_exact_duplicate_detected() {
    let meta = make_meta("a.jpg", 100, 100, 1000, "abcdef1234567890", None);
    let mut seen = HashSet::new();
    seen.insert("abcdef1234567890".to_string());
    let result = StageProcessor::exact_duplicate(&meta, &seen);
    assert!(!result.passed);
    assert_eq!(result.destination, Some("duplicates/exact".to_string()));
}

#[test]
fn test_exact_duplicate_unique() {
    let meta = make_meta("a.jpg", 100, 100, 1000, "abc123", None);
    let seen = HashSet::new();
    let result = StageProcessor::exact_duplicate(&meta, &seen);
    assert!(result.passed);
    assert_eq!(result.destination, None);
}

#[test]
fn test_perceptual_duplicate_detected() {
    let meta = make_meta("a.jpg", 100, 100, 1000, "abc", None);
    let dupes = vec!["/test/b.jpg".to_string()];
    let result = StageProcessor::perceptual_duplicate(&meta, &dupes);
    assert!(!result.passed);
    assert_eq!(
        result.destination,
        Some("duplicates/perceptual".to_string())
    );
}

#[test]
fn test_perceptual_duplicate_unique() {
    let meta = make_meta("a.jpg", 100, 100, 1000, "abc", None);
    let dupes: Vec<String> = vec![];
    let result = StageProcessor::perceptual_duplicate(&meta, &dupes);
    assert!(result.passed);
    assert_eq!(result.destination, None);
}

#[test]
fn test_tiny_image_below_threshold() {
    let meta = make_meta("tiny.png", 50, 50, 500, "x", None);
    let result = StageProcessor::tiny_image(&meta, 100, 100);
    assert!(!result.passed);
    assert_eq!(result.destination, Some("rejected/tiny".to_string()));
}

#[test]
fn test_tiny_image_above_threshold() {
    let meta = make_meta("normal.png", 1920, 1080, 50000, "x", None);
    let result = StageProcessor::tiny_image(&meta, 100, 100);
    assert!(result.passed);
    assert_eq!(result.destination, None);
}

#[test]
fn test_icon_detection_icon() {
    let meta = make_meta("icon.png", 64, 64, 1000, "x", None);
    let result = StageProcessor::icon_detection(&meta);
    assert!(!result.passed);
    assert_eq!(result.destination, Some("categories/icons".to_string()));
    assert_eq!(result.category, Some("icon".to_string()));
}

#[test]
fn test_icon_detection_not_icon() {
    let meta = make_meta("photo.png", 1920, 1080, 50000, "x", None);
    let result = StageProcessor::icon_detection(&meta);
    assert!(result.passed);
    assert_eq!(result.destination, None);
}

#[test]
fn test_thumbnail_detection_by_name() {
    let meta = make_meta("thumb_001.jpg", 640, 480, 10000, "x", None);
    let result = StageProcessor::thumbnail_detection(&meta);
    assert!(!result.passed);
    assert_eq!(
        result.destination,
        Some("categories/thumbnails".to_string())
    );
}

#[test]
fn test_thumbnail_detection_by_size() {
    let meta = make_meta("preview.jpg", 320, 240, 5000, "x", None);
    let result = StageProcessor::thumbnail_detection(&meta);
    assert!(!result.passed);
}

#[test]
fn test_thumbnail_detection_normal() {
    let meta = make_meta("photo.jpg", 1920, 1080, 50000, "x", None);
    let result = StageProcessor::thumbnail_detection(&meta);
    assert!(result.passed);
}

#[test]
fn test_screenshot_detection_common_resolution() {
    let meta = make_meta("shot.png", 1920, 1080, 100000, "x", None);
    let result = StageProcessor::screenshot_detection(&meta);
    assert!(!result.passed);
    assert_eq!(
        result.destination,
        Some("categories/screenshots".to_string())
    );
}

#[test]
fn test_screenshot_detection_by_name() {
    let meta = make_meta("Screenshot 2024-01-01.png", 1440, 900, 80000, "x", None);
    let result = StageProcessor::screenshot_detection(&meta);
    assert!(!result.passed);
}

#[test]
fn test_screenshot_detection_normal() {
    let meta = make_meta("photo.jpg", 2048, 1536, 200000, "x", None);
    let result = StageProcessor::screenshot_detection(&meta);
    assert!(result.passed);
}

#[test]
fn test_wallpaper_detection_ultrawide() {
    let meta = make_meta("wide.jpg", 3440, 1440, 300000, "x", None);
    let result = StageProcessor::wallpaper_detection(&meta);
    assert!(!result.passed);
    assert_eq!(
        result.destination,
        Some("categories/wallpapers".to_string())
    );
}

#[test]
fn test_wallpaper_detection_normal() {
    let meta = make_meta("photo.jpg", 1920, 1080, 100000, "x", None);
    let result = StageProcessor::wallpaper_detection(&meta);
    assert!(result.passed);
}

#[test]
fn test_document_detection_document() {
    let meta = make_meta("scan.png", 1700, 1200, 500000, "x", None);
    let result = StageProcessor::document_detection(&meta);
    assert!(!result.passed);
    assert_eq!(result.destination, Some("categories/documents".to_string()));
}

#[test]
fn test_document_detection_not_document() {
    let meta = make_meta("photo.jpg", 1920, 1080, 100000, "x", None);
    let result = StageProcessor::document_detection(&meta);
    assert!(result.passed);
}

#[test]
fn test_ai_classification_portrait() {
    let meta = make_meta("portrait.jpg", 1080, 1920, 200000, "x", None);
    let result = StageProcessor::ai_classification(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("portrait".to_string()));
}

#[test]
fn test_ai_classification_panorama() {
    let meta = make_meta("pano.jpg", 5000, 1500, 500000, "x", None);
    let result = StageProcessor::ai_classification(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("panorama".to_string()));
}

#[test]
fn test_ai_classification_social() {
    let meta = make_meta("social.png", 1000, 1000, 100000, "x", None);
    let result = StageProcessor::ai_classification(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("social".to_string()));
}

#[test]
fn test_ai_classification_low_quality() {
    let meta = make_meta("small.png", 200, 150, 500, "x", None);
    let result = StageProcessor::ai_classification(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("low_quality".to_string()));
}

#[test]
fn test_ai_classification_uncategorized() {
    let meta = make_meta("generic.gif", 800, 600, 50000, "x", None);
    let result = StageProcessor::ai_classification(&meta);
    assert!(result.passed);
    assert!(result.category.is_some());
}

#[test]
fn test_quality_ranking_excellent() {
    let meta = make_meta("excellent.jpg", 10000, 8000, 40000000, "x", None);
    let result = StageProcessor::quality_ranking(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("excellent".to_string()));
    assert!(result.score.unwrap_or(0.0) >= 90.0);
}

#[test]
fn test_quality_ranking_below_average() {
    let meta = make_meta("low.jpg", 80, 100, 100, "x", None);
    let result = StageProcessor::quality_ranking(&meta);
    assert!(result.passed);
    assert_eq!(result.category, Some("below_average".to_string()));
}

#[test]
fn test_domain_error_from() {
    let err = DomainError::NotFound("file.jpg".to_string());
    let s: String = err.into();
    assert_eq!(s, "Not found: file.jpg");
}

#[test]
fn test_pipeline_config_default() {
    let cfg = PipelineConfig::default();
    assert_eq!(cfg.hamming_threshold, 4);
    assert_eq!(cfg.min_width, 100);
    assert_eq!(cfg.min_height, 100);
    assert!(cfg.detect_icons);
    assert!(cfg.classification_enabled);
}

#[test]
fn test_duplicate_detector_no_duplicates() {
    let mut detector = DuplicateDetector::new(4);
    let r1 = detector.add("/a.jpg".to_string(), 0x1234567890ABCDEF);
    assert!(r1.is_empty());
    let r2 = detector.add("/b.jpg".to_string(), 0xFFFFFFFFFFFFFFFF);
    assert!(r2.is_empty());
}

#[test]
fn test_duplicate_detector_finds_duplicate() {
    let mut detector = DuplicateDetector::new(4);
    detector.add("/a.jpg".to_string(), 0x1234567890ABCDEF);
    let r2 = detector.add("/b.jpg".to_string(), 0x1234567890ABCDEF);
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0], "/a.jpg");
}

#[test]
fn test_duplicate_detector_clear() {
    let mut detector = DuplicateDetector::new(4);
    detector.add("/a.jpg".to_string(), 0x1234567890ABCDEF);
    detector.clear();
    let r = detector.add("/b.jpg".to_string(), 0x1234567890ABCDEF);
    assert!(r.is_empty());
}

#[test]
fn test_duplicate_detector_threshold_boundary() {
    let mut detector = DuplicateDetector::new(0);
    detector.add("/a.jpg".to_string(), 0xFFFF);
    let r = detector.add("/b.jpg".to_string(), 0xFFFE);
    assert!(
        r.is_empty(),
        "should not match when threshold=0 but bits differ"
    );
}
