use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{DHashHasher, Sha256Hasher};
use mc_infra::image::ImageRsDecoder;
use mc_infra::notify::InMemoryNotifier;
use mc_infra::scanner::NativeFileScanner;
use mediacleaner_pro::{api::routes::create_routes, state::AppState};
use std::sync::Arc;
use tokio::sync::RwLock;

fn make_png(path: &std::path::Path, w: u32, h: u32) {
    let buf = image::RgbImage::new(w, h);
    buf.save(path).expect("failed to create test PNG");
}

fn make_png_solid(path: &std::path::Path, w: u32, h: u32, r: u8, g: u8, b: u8) {
    use image::Pixel;
    let mut buf = image::RgbImage::new(w, h);
    for pixel in buf.pixels_mut() {
        *pixel = image::Rgb([r, g, b]);
    }
    buf.save(path).expect("failed to create solid PNG");
}

fn setup_state(tmp: &std::path::Path) -> Arc<RwLock<AppState>> {
    let config = mediacleaner_pro::config::Config {
        server_host: "127.0.0.1".to_string(),
        server_port: 0,
        source_dir: tmp.join("source").to_string_lossy().to_string(),
        dest_dir: tmp.join("dest").to_string_lossy().to_string(),
        hamming_threshold: 4,
        min_width: 1,
        min_height: 1,
        worker_threads: 0,
        db_path: tmp.join("test.db").to_string_lossy().to_string(),
        temporal_host: None,
        temporal_namespace: "default".to_string(),
        temporal_task_queue: "test".to_string(),
        supabase_url: None,
        supabase_key: None,
    };

    let fs = Arc::new(NativeFileSystem::new(tmp));
    let hasher = Arc::new(Sha256Hasher::new());
    let dhash = Arc::new(DHashHasher::new());
    let decoder = Arc::new(ImageRsDecoder::new());
    let notifier = Arc::new(InMemoryNotifier::new());
    let scanner = Arc::new(NativeFileScanner);

    Arc::new(RwLock::new(AppState::new(
        config, fs, scanner, hasher, dhash, decoder, notifier,
    )))
}

async fn wait_for_completion(client: &reqwest::Client, base: &str, timeout: std::time::Duration) {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > timeout {
            panic!("timed out waiting for pipeline to complete");
        }
        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        if !body["is_running"].as_bool().unwrap_or(true) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

async fn run_test<F, Fut>(f: F)
where
    F: FnOnce(u16, reqwest::Client) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let tmp = std::env::temp_dir().join("mc_pipeline_test");
    let _ = std::fs::create_dir_all(&tmp);
    let state = setup_state(&tmp);
    let router = create_routes(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    f(port, client).await;

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_pipeline_processes_single_image() {
    run_test(|port, client| async move {
        let src = std::env::temp_dir().join("mc_pipeline_test").join("source");
        let _ = std::fs::create_dir_all(&src);
        make_png(&src.join("photo.png"), 640, 480);

        let base = format!("http://127.0.0.1:{}", port);
        client
            .post(format!("{}/api/start", base))
            .json(&serde_json::json!({
                "source_dir": src.to_string_lossy().to_string(),
                "dest_dir": std::env::temp_dir()
                    .join("mc_pipeline_test")
                    .join("dest")
                    .to_string_lossy()
                    .to_string(),
            }))
            .send()
            .await
            .unwrap();

        wait_for_completion(&client, &base, std::time::Duration::from_secs(30)).await;

        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(!body["is_running"].as_bool().unwrap_or(true));
        assert_eq!(body["stats"]["error_count"], 0);
    })
    .await;
}

#[tokio::test]
async fn test_pipeline_empty_source() {
    run_test(|port, client| async move {
        let src = std::env::temp_dir()
            .join("mc_pipeline_test")
            .join("empty_source");
        let _ = std::fs::create_dir_all(&src);

        let base = format!("http://127.0.0.1:{}", port);
        client
            .post(format!("{}/api/start", base))
            .json(&serde_json::json!({
                "source_dir": src.to_string_lossy().to_string(),
                "dest_dir": std::env::temp_dir()
                    .join("mc_pipeline_test")
                    .join("dest")
                    .to_string_lossy()
                    .to_string(),
            }))
            .send()
            .await
            .unwrap();

        wait_for_completion(&client, &base, std::time::Duration::from_secs(5)).await;

        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(!body["is_running"].as_bool().unwrap_or(true));
    })
    .await;
}

#[tokio::test]
async fn test_pipeline_exact_duplicate() {
    run_test(|port, client| async move {
        let src = std::env::temp_dir()
            .join("mc_pipeline_test")
            .join("dup_source");
        let _ = std::fs::create_dir_all(&src);
        make_png(&src.join("original.png"), 100, 100);
        std::fs::copy(src.join("original.png"), src.join("copy.png")).unwrap();

        let base = format!("http://127.0.0.1:{}", port);
        client
            .post(format!("{}/api/start", base))
            .json(&serde_json::json!({
                "source_dir": src.to_string_lossy().to_string(),
                "hamming_threshold": 4,
                "dest_dir": std::env::temp_dir()
                    .join("mc_pipeline_test")
                    .join("dest")
                    .to_string_lossy()
                    .to_string(),
            }))
            .send()
            .await
            .unwrap();

        wait_for_completion(&client, &base, std::time::Duration::from_secs(30)).await;

        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(!body["is_running"].as_bool().unwrap_or(true));
        assert_eq!(body["stats"]["unique_count"], 1);
        assert_eq!(body["stats"]["duplicate_count"], 1);
        assert_eq!(body["stats"]["error_count"], 0);
    })
    .await;
}

#[tokio::test]
async fn test_pipeline_cancellation() {
    run_test(|port, client| async move {
        let src = std::env::temp_dir()
            .join("mc_pipeline_test")
            .join("cancel_source");
        let _ = std::fs::create_dir_all(&src);
        for i in 0..50 {
            make_png(&src.join(format!("img_{}.png", i)), 640, 480);
        }

        let base = format!("http://127.0.0.1:{}", port);
        client
            .post(format!("{}/api/start", base))
            .json(&serde_json::json!({
                "source_dir": src.to_string_lossy().to_string(),
                "dest_dir": std::env::temp_dir()
                    .join("mc_pipeline_test")
                    .join("dest")
                    .to_string_lossy()
                    .to_string(),
            }))
            .send()
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        client
            .post(format!("{}/api/control", base))
            .json(&serde_json::json!({"action": "cancel"}))
            .send()
            .await
            .unwrap();

        wait_for_completion(&client, &base, std::time::Duration::from_secs(5)).await;

        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(!body["is_running"].as_bool().unwrap_or(true));
    })
    .await;
}

#[tokio::test]
async fn test_pipeline_sse_progress() {
    run_test(|port, client| async move {
        let src = std::env::temp_dir()
            .join("mc_pipeline_test")
            .join("sse_source");
        let _ = std::fs::create_dir_all(&src);
        make_png(&src.join("img1.png"), 640, 480);
        make_png(&src.join("img2.png"), 320, 240);

        let base = format!("http://127.0.0.1:{}", port);

        client
            .post(format!("{}/api/start", base))
            .json(&serde_json::json!({
                "source_dir": src.to_string_lossy().to_string(),
                "dest_dir": std::env::temp_dir()
                    .join("mc_pipeline_test")
                    .join("dest")
                    .to_string_lossy()
                    .to_string(),
            }))
            .send()
            .await
            .unwrap();

        let resp = client
            .get(format!("{}/api/progress", base))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        wait_for_completion(&client, &base, std::time::Duration::from_secs(30)).await;

        let resp = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(!body["is_running"].as_bool().unwrap_or(true));
        assert_eq!(body["stats"]["error_count"], 0);
    })
    .await;
}
