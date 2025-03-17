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
    pub fn start_date_week_day(&self) -> String {
        self.start_date.format("%A").to_string()
    }

    // Method to check if start_date is today
    pub fn start_date_day(&self) -> String {
        self.start_date.format("%d").to_string()
    }

    // Method to check if start_date is today
    pub fn start_date_month(&self) -> String {
        self.start_date.format("%m").to_string()
    }

    pub fn end_date_week_day(&self) -> Option<String> {
        self.end_date
            .map(|end_date| end_date.format("%A").to_string())
    }

    pub fn end_date_day(&self) -> Option<String> {
        self.end_date
            .map(|end_date| end_date.format("%d").to_string())
    }

    pub fn end_date_month(&self) -> Option<String> {
        self.end_date
            .map(|end_date| end_date.format("%m").to_string())
    }

    pub fn is_day_event(&self) -> bool {
        if let Some(end_date) = self.end_date {
            self.start_date == end_date
        } else {
            // If item does not have end it is most likely one day event
            true
        }
    }

    pub fn number_of_days(&self) -> i64 {
        match self.end_date {
            Some(end_date) => {
                let days = end_date - self.start_date;
                days.num_days()
            }
            None => 0,
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

async fn download(url: &str, calendar_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Make the request to the WebCal URL
    let response = reqwest::get(url).await?;

    // Check if the response was successful
    if response.status().is_success() {
        // Get the bytes of the .ics file
        let calendar_data = response.bytes().await?;

        // Create or open the file where the calendar will be saved
        let mut file = File::create(calendar_path)?;

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
    calendar_path: &str,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    download(url, calendar_path).await?;

    let contents = read_to_string(calendar_path)?;

    let parsed_calendar: Calendar = contents.parse()?;

    let items = filter_events(&parsed_calendar, date, offset);

    Ok(items)
}
