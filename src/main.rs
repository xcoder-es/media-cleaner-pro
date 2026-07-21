use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{DHashHasher, Sha256Hasher};
use mc_infra::image::ImageRsDecoder;
use mc_infra::notify::InMemoryNotifier;
use mc_infra::scanner::NativeFileScanner;
use mc_infra::sqlite::SqliteJobRepo;
use mediacleaner_pro::{api::routes::create_routes, config::Config, state::AppState};

const DEFAULT_ENV_TEMPLATE: &str = r#"# MediaCleaner Pro Configuration
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Paths
SOURCE_DIR=./data/source
DEST_DIR=./data/output

# Processing
HAMMING_THRESHOLD=4
MIN_WIDTH=100
MIN_HEIGHT=100
WORKER_THREADS=0

# Temporal (optional)
TEMPORAL_HOST=localhost:7233
TEMPORAL_NAMESPACE=default
TEMPORAL_TASK_QUEUE=mediacleaner

# Supabase (optional)
SUPABASE_URL=
SUPABASE_KEY=
"#;

fn auto_init() {
    let dotenv_path = std::path::Path::new(".env");
    if !dotenv_path.exists() {
        if let Err(e) = std::fs::write(dotenv_path, DEFAULT_ENV_TEMPLATE) {
            eprintln!("Warning: could not create .env: {}", e);
        } else {
            let abs = std::fs::canonicalize(dotenv_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".env".to_string());
            println!(
                "First run — created .env at {abs}. Edit it to configure source/dest directories."
            );
        }
    }

    let dirs = ["./data/source", "./data/output"];
    for dir in dirs {
        let path = std::path::Path::new(dir);
        if !path.exists()
            && let Err(e) = std::fs::create_dir_all(path)
        {
            eprintln!("Warning: could not create {dir}: {e}");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    auto_init();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mediacleaner_pro=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    tracing::info!("Starting MediaCleaner Pro v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Author: Carlos Pinto <capintobe@gmail.com>");
    tracing::info!("Server: {}:{}", config.server_host, config.server_port);

    let file_system = Arc::new(NativeFileSystem::new(std::path::PathBuf::from(
        &config.source_dir,
    )));
    let exact_hasher = Arc::new(Sha256Hasher::new());
    let image_hasher = Arc::new(DHashHasher::new());
    let image_decoder = Arc::new(ImageRsDecoder::new());
    let notifier = Arc::new(InMemoryNotifier::new());
    let file_scanner = Arc::new(NativeFileScanner);

    if let Ok(repo) = SqliteJobRepo::new(&config.db_path) {
        tracing::info!("Job repository initialized at {}", config.db_path);
        drop(repo);
    }

    let app_state = Arc::new(RwLock::new(AppState::new(
        config.clone(),
        file_system,
        file_scanner,
        exact_hasher,
        image_hasher,
        image_decoder,
        notifier,
    )));

    if let Ok(db) = mediacleaner_pro::state::db::Database::new(&config.db_path) {
        tracing::info!("Database initialized at {}", config.db_path);
        drop(db);
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_routes(app_state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;

    tracing::info!(
        "MediaCleaner Pro ready at http://localhost:{}",
        config.server_port
    );
    tracing::info!(
        "Open http://localhost:{}/ in your browser to start processing",
        config.server_port
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
