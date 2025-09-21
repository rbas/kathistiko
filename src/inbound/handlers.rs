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
    sensor::{
        model::{LivingRoom, Outdoor},
        repository::repo::load_sensors_data,
    },
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
    pub living_room: Option<LivingRoom>,
    pub outdoor: Option<Outdoor>,
}

impl CalendarTemplate {
    pub fn get_month_name(&self) -> String {
        format!("{}", self.current_date.format("%B"))
    }

    pub fn get_smart_month_name(&self) -> String {
        let full_month = format!("{}", self.current_date.format("%B"));
        match full_month.as_str() {
            "January" => "Jan".to_string(),
            "February" => "Feb".to_string(),
            "August" => "Aug".to_string(),
            "September" => "Sep".to_string(),
            "October" => "Oct".to_string(),
            "November" => "Nov".to_string(),
            "December" => "Dec".to_string(),
            // Keep these full as they're short enough
            "March" | "April" | "May" | "June" | "July" => full_month,
            // Fallback for any other months
            _ => full_month,
        }
    }

    pub fn living_room_temperature(&self) -> String {
        match &self.living_room {
            Some(room) => format!("{:05.2}", room.temperature()),
            None => "".to_string(),
        }
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

    let result = load_sensors_data(
        state.settings.prometheus_url.as_str(),
        state.settings.prometheus_username.as_str(),
        state.settings.prometheus_password.as_str(),
    )
    .await;

    let mut template = CalendarTemplate {
        current_date: today,
        periodical_items,
        calendar_items,
        living_room: None,
        outdoor: None,
    };

    match result {
        Ok((living_room, outdoor)) => {
            template.living_room = living_room;
            template.outdoor = outdoor;
        }
        Err(err) => {
            error!("Cannot read data from temperature sensors {:#?}", err);
        }
    }

    HtmlTemplate(template)
}

pub async fn index() -> String {
    "OK".to_string()
}
