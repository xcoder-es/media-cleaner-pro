use mc_core::*;
use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{DHashHasher, Sha256Hasher};
use mc_infra::image::ImageRsDecoder;
use std::path::Path;

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

fn create_test_png(path: &Path, width: u32, height: u32) {
    let mut buf = image::RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let pixel = image::Rgb([(x * 255 / width) as u8, (y * 255 / height) as u8, 128]);
            buf.put_pixel(x, y, pixel);
        }
    }
    buf.save(path).expect("failed to create test PNG");
}

#[test]
fn test_native_fs_read_file() {
    let tmp = std::env::temp_dir().join("mc_infra_test_read");
    let _ = std::fs::create_dir_all(&tmp);
    let file_path = tmp.join("test.txt");
    std::fs::write(&file_path, b"hello world").unwrap();

    let fs = NativeFileSystem::new(&tmp);
    let data = rt().block_on(fs.read_file(&file_path)).unwrap();
    assert_eq!(data, b"hello world");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_write_file() {
    let tmp = std::env::temp_dir().join("mc_infra_test_write");
    let _ = std::fs::create_dir_all(&tmp);
    let fs = NativeFileSystem::new(&tmp);
    let file_path = tmp.join("out.txt");

    rt().block_on(fs.write_file(&file_path, b"test data"))
        .unwrap();
    let content = std::fs::read(&file_path).unwrap();
    assert_eq!(content, b"test data");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_move_file() {
    let tmp = std::env::temp_dir().join("mc_infra_test_move");
    let _ = std::fs::create_dir_all(&tmp);
    let src = tmp.join("src.txt");
    let dst = tmp.join("dst.txt");
    std::fs::write(&src, b"move me").unwrap();

    let fs = NativeFileSystem::new(&tmp);
    rt().block_on(fs.move_file(&src, &dst)).unwrap();
    assert!(!src.exists());
    assert!(dst.exists());
    assert_eq!(std::fs::read(&dst).unwrap(), b"move me");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_copy_file() {
    let tmp = std::env::temp_dir().join("mc_infra_test_copy");
    let _ = std::fs::create_dir_all(&tmp);
    let src = tmp.join("src.txt");
    let dst = tmp.join("dst.txt");
    std::fs::write(&src, b"copy me").unwrap();

    let fs = NativeFileSystem::new(&tmp);
    rt().block_on(fs.copy_file(&src, &dst)).unwrap();
    assert!(src.exists());
    assert!(dst.exists());
    assert_eq!(std::fs::read(&dst).unwrap(), b"copy me");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_delete_file() {
    let tmp = std::env::temp_dir().join("mc_infra_test_delete");
    let _ = std::fs::create_dir_all(&tmp);
    let path = tmp.join("todelete.txt");
    std::fs::write(&path, b"delete me").unwrap();

    let fs = NativeFileSystem::new(&tmp);
    rt().block_on(fs.delete_file(&path)).unwrap();
    assert!(!path.exists());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_create_dir() {
    let tmp = std::env::temp_dir().join("mc_infra_test_create_dir");
    let _ = std::fs::create_dir_all(&tmp);
    let sub = tmp.join("nested").join("sub").join("dir");

    let fs = NativeFileSystem::new(&tmp);
    rt().block_on(fs.create_dir(&sub)).unwrap();
    assert!(sub.exists());
    assert!(sub.is_dir());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_native_fs_canonicalize() {
    let tmp = std::env::temp_dir().join("mc_infra_test_canon");
    let _ = std::fs::create_dir_all(&tmp);
    let path = tmp.join("file.txt");
    std::fs::write(&path, b"x").unwrap();

    let fs = NativeFileSystem::new(&tmp);
    let result = rt().block_on(fs.canonicalize(&path)).unwrap();
    assert!(result.contains("mc_infra_test_canon"));
    assert!(result.contains("file.txt"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_sha256_hasher_known_input() {
    let hasher = Sha256Hasher::new();
    let hash = hasher.compute_sha256(b"hello").unwrap();
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_sha256_hasher_empty_input() {
    let hasher = Sha256Hasher::new();
    let hash = hasher.compute_sha256(b"").unwrap();
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_dhash_hasher_returns_non_zero_for_image() {
    let mut buf = image::RgbImage::new(16, 16);
    for x in 0..16 {
        for y in 0..16 {
            buf.put_pixel(x, y, image::Rgb([x as u8 * 16, y as u8 * 16, 128]));
        }
    }
    let mut bytes: Vec<u8> = Vec::new();
    buf.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let hasher = DHashHasher::new();
    let hash = hasher.compute_dhash(&bytes).unwrap();
    assert_ne!(hash, 0, "dhash should be non-zero for varied image");

    let same_hash = hasher.compute_dhash(&bytes).unwrap();
    assert_eq!(hash, same_hash, "dhash must be deterministic");
}

#[test]
fn test_image_decoder_returns_correct_dimensions() {
    let tmp = std::env::temp_dir().join("mc_infra_test_decoder.png");
    create_test_png(&tmp, 64, 48);

    let data = std::fs::read(&tmp).unwrap();
    let decoder = ImageRsDecoder::new();
    let info = decoder.decode(&data).unwrap();
    assert_eq!(info.width, 64);
    assert_eq!(info.height, 48);

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_dhash_hasher_deterministic() {
    let tmp = std::env::temp_dir().join("mc_infra_test_dhash.png");
    create_test_png(&tmp, 32, 32);
    let data = std::fs::read(&tmp).unwrap();

    let hasher = DHashHasher::new();
    let h1 = hasher.compute_dhash(&data).unwrap();
    let h2 = hasher.compute_dhash(&data).unwrap();
    assert_eq!(h1, h2);

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_hamming_distance_trait() {
    let hasher = DHashHasher::new();
    let d = hasher.hamming_distance(0xFF00, 0x00FF);
    assert_eq!(d, 16);
}
