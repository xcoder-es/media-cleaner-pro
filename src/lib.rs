pub mod api;
pub mod config;
pub mod processing;
pub mod state;
pub mod temporal;

use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<state::AppState>>;
