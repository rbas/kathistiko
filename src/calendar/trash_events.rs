use chrono::{Datelike, Local, NaiveDate, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashTaskStatus {
    Today,
    Tomorrow,
    ThisWeek,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PeriodicalItem {
    pub summary: String,
    pub start_date: NaiveDate,
}

impl PeriodicalItem {
    fn new(summary: String, start_date: NaiveDate) -> Self {
        Self {
            summary,
            start_date,
        }
    }
    // Method to get the name of the weekday for start_date
    pub fn week_day(&self) -> String {
        // Use NaiveDate's weekday() method, and map it to a string
        self.start_date.format("%A").to_string() // %A is the full weekday name (e.g., "Monday")
    }

    pub fn short_week_day(&self) -> String {
        self.start_date.format("%a").to_string().to_uppercase()
    }

    pub fn day(&self) -> u32 {
        self.start_date.day()
    }

    pub fn status_on(&self, today: NaiveDate) -> Option<TrashTaskStatus> {
        if self.start_date < today {
            None
        } else if self.start_date == today {
            Some(TrashTaskStatus::Today)
        } else if self.start_date == today + chrono::Duration::days(1) {
            Some(TrashTaskStatus::Tomorrow)
        } else if self.start_date.iso_week() == today.iso_week() {
            Some(TrashTaskStatus::ThisWeek)
        } else {
            None
        }
    }

    fn status(&self) -> Option<TrashTaskStatus> {
        self.status_on(Local::now().date_naive())
    }

    pub fn is_visible(&self) -> bool {
        self.status().is_some()
    }

    // Method to check if start_date is today
    pub fn is_today(&self) -> bool {
        self.status() == Some(TrashTaskStatus::Today)
    }

    // Method to check if start_date is tomorrow
    pub fn is_tomorrow(&self) -> bool {
        self.status() == Some(TrashTaskStatus::Tomorrow)
    }

    pub fn is_this_week(&self) -> bool {
        self.status() == Some(TrashTaskStatus::ThisWeek)
    }
}

pub fn generate_periodical_events(date: NaiveDate) -> Option<Vec<PeriodicalItem>> {
    let week_number = date.iso_week().week();

    let mut items = vec![];

    if week_number % 2 == 0 {
        let start_date = get_weekday_of_week(date, Weekday::Thu);
        let plastic = PeriodicalItem::new("General waste".to_string(), start_date);
        items.push(plastic);
    } else {
        let start_date = get_weekday_of_week(date, Weekday::Tue);
        let general = PeriodicalItem::new("Plastic waste".to_string(), start_date);

        items.push(general);
    }

    let paper_start_date = NaiveDate::from_ymd_opt(2025, 1, 16).unwrap();
    let paper_start_week = paper_start_date.iso_week().week();

    // Get the calendar week of the current date
    let current_week = date.iso_week().week();

    // Calculate the difference in weeks based on calendar weeks
    let weeks_since_start = if current_week >= paper_start_week {
        current_week - paper_start_week
    } else {
        // If the current week is before the paper_start_date, adjust accordingly.
        (current_week + 52) - paper_start_week
    };

    // Check if we are on the 4th week
    if weeks_since_start % 4 == 0 {
        let start_date = get_weekday_of_week(date, Weekday::Thu);
        let paper = PeriodicalItem::new("Paper waste".to_string(), start_date);
        items.push(paper)
    }

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

fn get_weekday_of_week(date: NaiveDate, weekday: Weekday) -> NaiveDate {
    let current_weekday = date.weekday();
    let days_diff =
        weekday.num_days_from_monday() as i64 - current_weekday.num_days_from_monday() as i64;

    date + chrono::Duration::days(days_diff)
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, Weekday};

    use crate::calendar::trash_events::{
        generate_periodical_events, get_weekday_of_week, PeriodicalItem, TrashTaskStatus,
    };

    #[test]
    fn generate_plastic_event() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 24).unwrap();

        let expected = PeriodicalItem::new(
            "Plastic waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 25).unwrap(),
        );
        let result = generate_periodical_events(date);

        assert!(result.is_some());

        let items = result.unwrap();
        assert!(items.len() == 1);
        assert_eq!(expected, items[0]);
    }

    #[test]
    fn generate_general_event() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 22).unwrap();

        let expected = PeriodicalItem::new(
            "General waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 20).unwrap(),
        );
        let result = generate_periodical_events(date);

        assert!(result.is_some());

        let items = result.unwrap();
        assert!(items.len() == 1);
        assert_eq!(expected, items[0]);
    }

    #[test]
    fn expect_only_general() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 17).unwrap();

        let expected = PeriodicalItem::new(
            "General waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 20).unwrap(),
        );
        let result = generate_periodical_events(date);

        assert!(result.is_some());

        let items = result.unwrap();
        assert!(items.len() == 1);
        assert_eq!(expected, items[0]);
    }

    #[test]
    fn expect_plastic_and_paper_event_is_future_year() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let plastic_item = PeriodicalItem::new(
            "Plastic waste".to_string(),
            NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
        );
        let paper_item = PeriodicalItem::new(
            "Paper waste".to_string(),
            NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
        );
        let result = generate_periodical_events(date);

        assert!(result.is_some());

        let items = result.unwrap();
        assert!(items.len() == 2);
        assert_eq!(plastic_item, items[0]);
        assert_eq!(paper_item, items[1]);
    }

    #[test]
    fn generate_plastic_and_paper_event() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();

        let plastic_item = PeriodicalItem::new(
            "Plastic waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 11).unwrap(),
        );
        let paper_item = PeriodicalItem::new(
            "Paper waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 13).unwrap(),
        );
        let result = generate_periodical_events(date);

        assert!(result.is_some());

        let items = result.unwrap();
        assert!(items.len() == 2);
        assert_eq!(plastic_item, items[0]);
        assert_eq!(paper_item, items[1]);
    }

    #[test]
    fn expect_week_day_after_the_current_date() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 24).unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 3, 25).unwrap();
        let actual = get_weekday_of_week(date, Weekday::Tue);
        assert_eq!(expected, actual);
    }

    #[test]
    fn expect_week_day_before_the_current_date() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 3, 12).unwrap();
        let actual = get_weekday_of_week(date, Weekday::Wed);
        assert_eq!(expected, actual);
    }

    #[test]
    fn trash_task_moves_through_visible_statuses_then_disappears() {
        let task = PeriodicalItem::new(
            "General waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 20).unwrap(),
        );

        assert_eq!(
            task.status_on(NaiveDate::from_ymd_opt(2025, 3, 17).unwrap()),
            Some(TrashTaskStatus::ThisWeek)
        );
        assert_eq!(
            task.status_on(NaiveDate::from_ymd_opt(2025, 3, 19).unwrap()),
            Some(TrashTaskStatus::Tomorrow)
        );
        assert_eq!(
            task.status_on(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap()),
            Some(TrashTaskStatus::Today)
        );
        assert_eq!(
            task.status_on(NaiveDate::from_ymd_opt(2025, 3, 21).unwrap()),
            None
        );
    }

    #[test]
    fn trash_task_outside_current_week_is_hidden() {
        let task = PeriodicalItem::new(
            "Plastic waste".to_string(),
            NaiveDate::from_ymd_opt(2025, 3, 25).unwrap(),
        );

        assert_eq!(
            task.status_on(NaiveDate::from_ymd_opt(2025, 3, 17).unwrap()),
            None
        );
    }
}
