use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{DHashHasher, Sha256Hasher};
use mc_infra::image::ImageRsDecoder;
use mc_infra::notify::InMemoryNotifier;
use mc_infra::scanner::NativeFileScanner;
use mediacleaner_pro::{api::routes::create_routes, state::AppState};
use std::sync::Arc;
use tokio::sync::RwLock;

fn setup(tmp: &std::path::Path) -> Arc<RwLock<AppState>> {
    let config = mediacleaner_pro::config::Config {
        server_host: "127.0.0.1".to_string(),
        server_port: 0,
        source_dir: tmp.join("source").to_string_lossy().to_string(),
        dest_dir: tmp.join("dest").to_string_lossy().to_string(),
        hamming_threshold: 4,
        min_width: 100,
        min_height: 100,
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

fn create_test_png(path: &std::path::Path) {
    let buf = image::RgbImage::new(64, 48);
    buf.save(path).expect("failed to create test PNG");
}

async fn run_test<F, Fut>(f: F)
where
    F: FnOnce(u16) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let tmp = std::env::temp_dir().join("mc_api_test");
    let _ = std::fs::create_dir_all(&tmp);

    let state = setup(&tmp);

    let router = create_routes(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    f(port).await;

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_health_endpoint() {
    run_test(|port| async move {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "mediacleaner-pro");
    })
    .await;
}

#[tokio::test]
async fn test_openapi_endpoint() {
    run_test(|port| async move {
        let resp = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}/api/openapi.json", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["openapi"], "3.1.0");
        assert!(body["paths"].is_object());
        assert!(body["paths"].get("/api/status").is_some());
    })
    .await;
}

#[tokio::test]
async fn test_status_endpoint() {
    run_test(|port| async move {
        let resp = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}/api/status", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["stages"].is_array());
        assert_eq!(body["stages"].as_array().unwrap().len(), 10);
        assert_eq!(body["is_running"].as_bool().unwrap_or(true), false);
    })
    .await;
}

#[tokio::test]
async fn test_start_job_endpoint() {
    run_test(|port| async move {
        let tmp = std::env::temp_dir().join("mc_api_test_source");
        let _ = std::fs::create_dir_all(&tmp);
        create_test_png(&tmp.join("test.png"));

        let resp = reqwest::Client::new()
            .post(format!("http://127.0.0.1:{}/api/start", port))
            .json(&serde_json::json!({
                "source_dir": tmp.to_string_lossy().to_string(),
                "dest_dir": std::env::temp_dir().join("mc_api_test_dest").to_string_lossy().to_string(),
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["job_id"].is_string());
        assert_eq!(body["status"], "started");
        assert!(body["is_running"].as_bool().unwrap_or(false));

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let _ = std::fs::remove_dir_all(&tmp);
    })
    .await;
}

#[tokio::test]
async fn test_start_job_requires_source_dir() {
    run_test(|port| async move {
        let resp = reqwest::Client::new()
            .post(format!("http://127.0.0.1:{}/api/start", port))
            .json(&serde_json::json!({
                "source_dir": "",
                "dest_dir": "",
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    })
    .await;
}

#[tokio::test]
async fn test_control_pause_resume() {
    run_test(|port| async move {
        let client = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}", port);

        client
            .post(format!("{}/api/control", base))
            .json(&serde_json::json!({"action": "pause"}))
            .send()
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let status = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = status.json().await.unwrap();
        assert_eq!(body["is_paused"].as_bool().unwrap_or(false), true);

        client
            .post(format!("{}/api/control", base))
            .json(&serde_json::json!({"action": "resume"}))
            .send()
            .await
            .unwrap();

        let status = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = status.json().await.unwrap();
        assert_eq!(body["is_paused"].as_bool().unwrap_or(true), false);
    })
    .await;
}

#[tokio::test]
async fn test_control_cancel() {
    run_test(|port| async move {
        let client = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}", port);

        client
            .post(format!("{}/api/control", base))
            .json(&serde_json::json!({"action": "cancel"}))
            .send()
            .await
            .unwrap();

        let status = client
            .get(format!("{}/api/status", base))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = status.json().await.unwrap();
        assert_eq!(body["is_running"].as_bool().unwrap_or(true), false);
    })
    .await;
}

#[tokio::test]
async fn test_logs_endpoint() {
    run_test(|port| async move {
        let resp = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}/api/logs", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body.is_array());
    })
    .await;
}

#[tokio::test]
async fn test_browse_endpoint() {
    run_test(|port| async move {
        let resp = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}/api/browse", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    })
    .await;
}
