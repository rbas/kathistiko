use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use reqwest::StatusCode;

use super::http::AppState;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "calendar.html")]
pub struct CalendarTemplate {}

pub async fn calendar_handler(State(state): State<AppState>) -> impl IntoResponse {
    let template = CalendarTemplate {};

    HtmlTemplate(template)
}
