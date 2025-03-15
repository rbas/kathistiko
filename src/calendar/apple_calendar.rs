use std::{
    fs::{read_to_string, File},
    io::Write,
};

use chrono::{Days, NaiveDate};
use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime, Event};

#[derive(Debug)]
pub struct Item {
    pub summary: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

impl Item {
    fn new(summary: String, start_date: NaiveDate, end_date: Option<NaiveDate>) -> Self {
        Self {
            summary,
            start_date,
            end_date,
        }
    }
}

impl From<&Event> for Item {
    fn from(event: &Event) -> Self {
        let summary = event.get_summary().unwrap_or("Summary wasn't filled");
        let start_date = event.get_start().unwrap().date_naive();

        let mut end_date: Option<NaiveDate> = None;

        if let Some(end) = event.get_end() {
            end_date = Some(end.date_naive());
        }

        Self::new(summary.to_string(), start_date, end_date)
    }
}

async fn download(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // let url = "https://p114-caldav.icloud.com/published/2/MTE3OTA4NTk4NTgxMTc5MJObCer765x2961wzsaODjbNjUqDqKwAqv14wWt44uuvywEQKiilkxtMjkoaREvYWMdwuH3hbelMjF1172MzQNU"; // Replace with your WebCal URL
    // let url = "http://p114-caldav.icloud.com/published/2/MTE3OTA4NTk4NTgxMTc5MJObCer765x2961wzsaODjbUjEvPjLBMgv_O6bi0BWbBpyj13Be93p8OK3g89Bd4ufctX_NG8pGVCEaLfoDpawY";

    // Make the request to the WebCal URL
    let response = reqwest::get(url).await?;

    // Check if the response was successful
    if response.status().is_success() {
        // Get the bytes of the .ics file
        let calendar_data = response.bytes().await?;

        // Create or open the file where the calendar will be saved
        let mut file = File::create("fixtures/family.ics")?;

        // Write the bytes to the file
        file.write_all(&calendar_data)?;

        println!("Calendar successfully downloaded!");
    } else {
        eprintln!(
            "Failed to fetch the calendar. Status: {}",
            response.status()
        );
    }

    Ok(())
}

fn is_actual_event(
    event_start: Option<DatePerhapsTime>,
    event_end: Option<DatePerhapsTime>,
    today: NaiveDate,
    start_date_offset: NaiveDate,
) -> bool {
    if event_start.is_none() {
        return false;
    }

    let start = event_start.unwrap().date_naive();

    if start >= today && start <= start_date_offset {
        return true;
    }

    match event_end {
        Some(date) => {
            let end = date.date_naive();

            start <= today && end >= today
        }
        None => false,
    }
}

fn filter_events(calendar: &Calendar, day: NaiveDate, offset: Days) -> Vec<Item> {
    let stop_day = day + offset;

    let mut items: Vec<Item> = Vec::new();
    for component in &calendar.components {
        if let CalendarComponent::Event(event) = component {
            if is_actual_event(event.get_start(), event.get_end(), day, stop_day) {
                items.push(Item::from(event));
            }
        }
    }
    items.sort_by(|a, b| a.start_date.cmp(&b.start_date));

    items
}

pub async fn get_calendar_items(
    url: &str,
    date: NaiveDate,
    offset: Days,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    download(url).await?;

    let contents = read_to_string("fixtures/family.ics")?;

    let parsed_calendar: Calendar = contents.parse()?;

    let items = filter_events(&parsed_calendar, date, offset);

    Ok(items)
}
