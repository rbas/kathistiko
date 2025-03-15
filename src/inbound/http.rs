use axum::Router;
use log::info;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use crate::settings::Settings;

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
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("output"))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&socket_addr).await?;

    info!("Server running at {}", socket_addr.to_string());
    // Run the server
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
