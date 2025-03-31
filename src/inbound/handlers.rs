use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Datelike, Days, Local};
use log::error;
use reqwest::StatusCode;

use crate::calendar::{
    apple_calendar::{get_calendar_items, Item},
    trash_events::{generate_periodical_events, PeriodicalItem},
};

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
pub struct CalendarTemplate {
    pub current_date: DateTime<Local>,
    pub periodical_items: Option<Vec<PeriodicalItem>>,
    pub calendar_items: Option<Vec<Item>>,
}

impl CalendarTemplate {
    pub fn get_month_name(&self) -> String {
        format!("{}", self.current_date.format("%B"))
    }
}

pub async fn calendar_handler(State(state): State<AppState>) -> impl IntoResponse {
    let today = Local::now();

    let date = today.date_naive();

    let periodical_items = generate_periodical_events(date);

    let url = state.settings.family_calendar_url.as_str();
    let offset = Days::new(state.settings.family_calendar_offset_days);

    let calendar_path = format!("{}/calendar.ics", &state.settings.tmp_folder);
    let result = get_calendar_items(url, date, offset, &calendar_path).await;

    let calendar_items = match result {
        Ok(items) => Some(items),
        Err(err) => {
            error!("Cannot read data from family calendar {:#?}", err);
            None
        }
    };

    let template = CalendarTemplate {
        current_date: today,
        periodical_items,
        calendar_items,
    };

    HtmlTemplate(template)
}

pub async fn index() -> String {
    "OK".to_string()
}
