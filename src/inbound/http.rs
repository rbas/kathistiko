use axum::{routing::get, Router};
use log::info;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use crate::{inbound::handlers::calendar_handler, settings::Settings};

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Settings,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

pub async fn spawn_web_server(
    socket_addr: SocketAddr,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let public_folder = state.settings.clone().public_folder;
    let app = Router::new()
        .route("/", get(calendar_handler))
        .route("/calendar", get(calendar_handler))
        .nest_service("/public", ServeDir::new(public_folder))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&socket_addr).await?;

    info!("Server running at {}", socket_addr.to_string());
    // Run the server
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
