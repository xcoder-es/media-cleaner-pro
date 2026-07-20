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

    let app_state = Arc::new(RwLock::new(AppState::new(config.clone())));

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
