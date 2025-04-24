use std::{
    fs::{metadata, read_to_string, File},
    hash::DefaultHasher,
    io::Write,
    path::Path,
    time::Duration,
};

use std::hash::{Hash, Hasher};

use chrono::{Days, NaiveDate, NaiveTime};
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, Event};

#[derive(Debug)]
pub struct Item {
    pub summary: String,
    pub start_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_date: Option<NaiveDate>,
    pub end_time: Option<NaiveTime>,
}

impl Item {
    fn new(
        summary: String,
        start_date: NaiveDate,
        start_time: Option<NaiveTime>,
        end_date: Option<NaiveDate>,
        end_time: Option<NaiveTime>,
    ) -> Self {
        Self {
            summary,
            start_date,
            start_time,
            end_date,
            end_time,
        }
    }
    pub fn start_date_week_day(&self) -> String {
        self.start_date.format("%A").to_string()
    }

    // Method to get start_date day
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
                && (self.start_time.is_none()
                    || self.end_time.is_none()
                    || self.start_time == self.end_time)
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

        let start = event.get_start().unwrap();
        let start_date = start.date_naive();
        let start_time = extract_time_if_present(&start);

        let (end_date, end_time) = if let Some(end) = event.get_end() {
            let raw_end_date = end.date_naive();

            // Adjust end date for all-day events (subtract one day)
            let adjusted_end_date =
                if start_time.is_none() && extract_time_if_present(&end).is_none() {
                    // For all-day events, subtract one day
                    raw_end_date.pred_opt().unwrap_or(raw_end_date)
                } else {
                    raw_end_date
                };

            (Some(adjusted_end_date), extract_time_if_present(&end))
        } else {
            (None, None)
        };

        Self::new(
            summary.to_string(),
            start_date,
            start_time,
            end_date,
            end_time,
        )
    }
}

fn extract_time_if_present(date_perhaps_time: &DatePerhapsTime) -> Option<NaiveTime> {
    match date_perhaps_time {
        DatePerhapsTime::DateTime(dt) => {
            // Extract time from the datetime
            match dt {
                CalendarDateTime::Floating(naive_dt) => Some(naive_dt.time()),
                CalendarDateTime::Utc(utc_dt) => Some(utc_dt.naive_utc().time()),
                CalendarDateTime::WithTimezone { date_time, tzid: _ } => Some(date_time.time()),
            }
        }
        DatePerhapsTime::Date(_) => None,
    }
}

fn is_cache_valid(cache_path: &str, ttl: u64) -> bool {
    if let Ok(metadata) = metadata(cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                return elapsed <= Duration::new(ttl, 0);
            }
        }
    }
    false
}

async fn download(
    url: &str,
    calendar_path: &str,
    cache_ttl: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(calendar_path).exists() && is_cache_valid(calendar_path, cache_ttl) {
        println!("Using cached calendar file.");
        return Ok(());
    }

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
    urls: Vec<String>,
    date: NaiveDate,
    offset: Days,
    cache_folder: &str,
    cache_ttl: u64,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let mut all_items: Vec<Item> = Vec::new();

    for url in urls.iter() {
        // Generate a unique file path based on the URL hash
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();
        let calendar_path = format!("{}/calendar_{}.ics", cache_folder, hash);

        // Download the calendar (with caching)
        download(url, &calendar_path, cache_ttl).await?;

        // Read the calendar file
        let contents = read_to_string(&calendar_path)?;

        // Parse the calendar
        let parsed_calendar: Calendar = contents.parse()?;

        // Filter events
        let items = filter_events(&parsed_calendar, date, offset);

        // Collect items
        all_items.extend(items);
    }

    // Sort all items by start_date
    all_items.sort_by(|a, b| a.start_date.cmp(&b.start_date));

    Ok(all_items)
}
