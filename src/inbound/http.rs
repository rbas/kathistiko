use std::sync::Arc;

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

use crate::{display::DisplayStore, inbound::handlers::calendar_handler, settings::Settings};

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub display: DisplayStore,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Arc::new(settings),
            display: DisplayStore::default(),
        }
    }
}

pub fn router(state: AppState) -> Router {
    let public_folder = state.settings.public_folder.clone();
    let output_folder = state.settings.display.output_directory.clone();

    Router::new()
        .route("/", get(calendar_handler))
        .route("/calendar", get(calendar_handler))
        .route("/latest", get(latest_bitmap_handler))
        .route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
        .route("/health/ready", get(readiness_handler))
        .nest_service("/public", ServeDir::new(public_folder))
        .nest_service("/assets", ServeDir::new(output_folder))
        .with_state(state)
}

async fn latest_bitmap_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    match state.display.latest().await {
        Some(bitmap) => (
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            bitmap,
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Display image is not ready",
        )
            .into_response(),
    }
}

async fn readiness_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> StatusCode {
    if !state.settings.display.enabled || state.display.latest().await.is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
