use std::sync::Arc;

use axum::{
    http::{header, HeaderMap, StatusCode},
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
    headers: HeaderMap,
) -> Response {
    match state.display.latest().await {
        Some(image) if if_none_match_matches(&headers, image.etag()) => (
            StatusCode::NOT_MODIFIED,
            [
                (header::CACHE_CONTROL, "no-cache".to_string()),
                (header::ETAG, image.etag().to_string()),
            ],
        )
            .into_response(),
        Some(image) => (
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (header::CACHE_CONTROL, "no-cache".to_string()),
                (header::ETAG, image.etag().to_string()),
            ],
            image.bitmap().to_vec(),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Display image is not ready",
        )
            .into_response(),
    }
}

fn if_none_match_matches(headers: &HeaderMap, current_etag: &str) -> bool {
    headers
        .get_all(header::IF_NONE_MATCH)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate.trim_start_matches("W/") == current_etag)
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

#[cfg(test)]
mod tests {
    use axum::http::{header, HeaderMap, HeaderValue};

    use super::if_none_match_matches;

    #[test]
    fn matches_current_etag_in_if_none_match() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("\"current\""),
        );

        assert!(if_none_match_matches(&headers, "\"current\""));
        assert!(!if_none_match_matches(&headers, "\"other\""));
    }

    #[test]
    fn supports_etag_lists_weak_validators_and_wildcard() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("\"old\", W/\"current\""),
        );
        assert!(if_none_match_matches(&headers, "\"current\""));

        headers.insert(header::IF_NONE_MATCH, HeaderValue::from_static("*"));
        assert!(if_none_match_matches(&headers, "\"anything\""));
    }
}
