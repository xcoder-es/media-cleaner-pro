use chrono::Utc;
use mc_core::*;
use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{DHashHasher, Sha256Hasher};
use mc_infra::image::ImageRsDecoder;
use mc_infra::sqlite::SqliteJobRepo;
use std::path::Path;
use std::path::PathBuf;

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

fn create_test_job(id: &str, user_id: Option<&str>, team_id: Option<&str>) -> Job {
    Job {
        id: JobId::from(id.to_string()),
        user_id: user_id.map(|u| UserId::from(u.to_string())),
        team_id: team_id.map(|t| TeamId::from(t.to_string())),
        source_dir: PathBuf::from("/tmp/src"),
        dest_dir: PathBuf::from("/tmp/dst"),
        config: PipelineConfig::default(),
        stages: Vec::new(),
        stats: ProcessingStats::default(),
        status: JobStatus::Pending,
        created_at: Utc::now(),
        completed_at: None,
        sync_status: SyncStatus::NotSynced,
    }
}

#[test]
fn test_sqlite_query_by_team_returns_correct_jobs() {
    let tmp = std::env::temp_dir().join("mc_infra_test_query_team.db");
    let _ = std::fs::remove_file(&tmp);

    let repo = SqliteJobRepo::new(&tmp).unwrap();
    let rt = rt();

    let team_a = TeamId::from("team-alpha".to_string());
    let team_b = TeamId::from("team-beta".to_string());

    let job1 = create_test_job("job1", Some("user1"), Some("team-alpha"));
    let job2 = create_test_job("job2", Some("user2"), Some("team-alpha"));
    let job3 = create_test_job("job3", Some("user1"), Some("team-beta"));

    rt.block_on(repo.create_job(&job1)).unwrap();
    rt.block_on(repo.create_job(&job2)).unwrap();
    rt.block_on(repo.create_job(&job3)).unwrap();

    let alpha_jobs = rt.block_on(repo.query_by_team(&team_a)).unwrap();
    assert_eq!(alpha_jobs.len(), 2, "team-alpha should have 2 jobs");
    assert!(alpha_jobs
        .iter()
        .any(|j| j.id == JobId::from("job1".to_string())));
    assert!(alpha_jobs
        .iter()
        .any(|j| j.id == JobId::from("job2".to_string())));

    let beta_jobs = rt.block_on(repo.query_by_team(&team_b)).unwrap();
    assert_eq!(beta_jobs.len(), 1, "team-beta should have 1 job");
    assert_eq!(beta_jobs[0].id, JobId::from("job3".to_string()));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_sqlite_list_jobs_all_vs_by_user() {
    let tmp = std::env::temp_dir().join("mc_infra_test_list_jobs.db");
    let _ = std::fs::remove_file(&tmp);

    let repo = SqliteJobRepo::new(&tmp).unwrap();
    let rt = rt();

    let uid = UserId::from("user-42".to_string());

    let job1 = create_test_job("a", Some("user-42"), None);
    let job2 = create_test_job("b", Some("user-42"), None);
    let job3 = create_test_job("c", Some("other"), None);

    rt.block_on(repo.create_job(&job1)).unwrap();
    rt.block_on(repo.create_job(&job2)).unwrap();
    rt.block_on(repo.create_job(&job3)).unwrap();

    let all = rt.block_on(repo.list_jobs(None, 100)).unwrap();
    assert_eq!(all.len(), 3, "list_jobs(None) should return all jobs");

    let user_jobs = rt.block_on(repo.list_jobs(Some(&uid), 100)).unwrap();
    assert_eq!(user_jobs.len(), 2, "user-42 should have 2 jobs");
    assert!(user_jobs.iter().all(|j| j.user_id == Some(uid.clone())));

    let _ = std::fs::remove_file(&tmp);
}
