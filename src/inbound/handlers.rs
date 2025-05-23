use askama::Template;
use axum::{
    debug_handler,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Datelike, Days, Local};
use log::error;
use reqwest::StatusCode;

use crate::{
    calendar::{
        apple_calendar::{get_calendar_items, Item},
        trash_events::{generate_periodical_events, PeriodicalItem},
    },
    sensor::temperature::{download_temperature_data, Sensor},
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
    pub temperature_data: Option<Vec<Sensor>>,
}

impl CalendarTemplate {
    pub fn get_month_name(&self) -> String {
        format!("{}", self.current_date.format("%B"))
    }
}

#[debug_handler]
pub async fn calendar_handler(State(state): State<AppState>) -> impl IntoResponse {
    let today = Local::now();

    let date = today.date_naive();

    let periodical_items = generate_periodical_events(date);

    let urls = state.settings.calendars_urls;
    let offset = Days::new(state.settings.calendar_offset_days);

    let cache_folder = state.settings.tmp_folder.as_str();
    let result = get_calendar_items(
        urls,
        date,
        offset,
        cache_folder,
        state.settings.cache_ttl_seconds,
    )
    .await;

    let calendar_items = match result {
        Ok(items) => Some(items),
        Err(err) => {
            error!("Cannot read data from family calendar {:#?}", err);
            None
        }
    };

    let result = download_temperature_data(
        state.settings.prometheus_url.as_str(),
        state.settings.prometheus_username.as_str(),
        state.settings.prometheus_password.as_str(),
    )
    .await;

    let temperature_data = match result {
        Ok(data) => match data.try_into() {
            Ok(sensors) => Some(sensors),
            Err(err) => {
                error!("Cannot parse temperature sensor data {:#?}", err);
                None
            }
        },
        Err(err) => {
            error!("Cannot get temperature sensor data {:#?}", err);
            None
        }
    };

    let template = CalendarTemplate {
        current_date: today,
        periodical_items,
        calendar_items,
        temperature_data,
    };

    HtmlTemplate(template)
}

pub async fn index() -> String {
    "OK".to_string()
}
