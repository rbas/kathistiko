use axum::extract::State;

use super::http::AppState;

pub async fn calendar_handler(State(state): State<AppState>) -> &'static str {
    "foo"
}
