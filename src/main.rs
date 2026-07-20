use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mediacleaner_pro::{
    api::routes::create_routes,
    config::Config,
    state::AppState,
};
use mc_infra::fs::NativeFileSystem;
use mc_infra::hash::{Sha256Hasher, DHashHasher};
use mc_infra::image::ImageRsDecoder;
use mc_infra::notify::InMemoryNotifier;
use mc_infra::sqlite::SqliteJobRepo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let file_system = Arc::new(NativeFileSystem::new(
        std::path::PathBuf::from(&config.source_dir),
    ));
    let exact_hasher = Arc::new(Sha256Hasher::new());
    let image_hasher = Arc::new(DHashHasher::new());
    let image_decoder = Arc::new(ImageRsDecoder::new());
    let notifier = Arc::new(InMemoryNotifier::new());

    if let Ok(repo) = SqliteJobRepo::new(&config.db_path) {
        tracing::info!("Job repository initialized at {}", config.db_path);
        drop(repo);
    }

    let app_state = Arc::new(RwLock::new(AppState::new(
        config.clone(),
        file_system,
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

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port)
        .parse()?;

    tracing::info!("MediaCleaner Pro ready at http://{}", addr);
    tracing::info!("Open the frontend to start processing");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
