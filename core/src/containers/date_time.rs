use std::{ops::Deref, str::FromStr};

use chrono::NaiveDateTime;

pub const DATETIME_FORMATS: &[&str] = &[
    // ISO 8601 formats (without timezone)
    "%Y-%m-%dT%H:%M:%S%.3f", // 2023-12-25T14:30:45.123
    "%Y-%m-%dT%H:%M:%S%.6f", // 2023-12-25T14:30:45.123456
    "%Y-%m-%dT%H:%M:%S",     // 2023-12-25T14:30:45
    "%Y-%m-%d %H:%M:%S",     // 2023-12-25 14:30:45
    // Written date formats
    "%a, %d %b %Y %H:%M:%S", // Mon, 25 Dec 2023 14:30:45
    "%d %b %Y %H:%M:%S",     // 25 Dec 2023 14:30:45
    "%a %b %d %H:%M:%S %Y",  // Mon Dec 25 14:30:45 2023
    "%b %d %H:%M:%S %Y",     // Dec 25 14:30:45 2023
    // US formats
    "%m/%d/%Y %H:%M:%S",    // 12/25/2023 14:30:45
    "%m/%d/%Y %I:%M:%S %p", // 12/25/2023 2:30:45 PM
    "%m-%d-%Y %H:%M:%S",    // 12-25-2023 14:30:45
    "%m-%d-%Y %I:%M:%S %p", // 12-25-2023 2:30:45 PM
    // "%m/%d/%Y %H:%M",       // 12/25/2023 14:30
    "%m/%d/%Y %I:%M %p", // 12/25/2023 2:30 PM
    // European formats
    "%d/%m/%Y %H:%M:%S", // 25/12/2023 14:30:45
    "%d.%m.%Y %H:%M:%S", // 25.12.2023 14:30:45
    "%d-%m-%Y %H:%M:%S", // 25-12-2023 14:30:45
    "%d %m %Y %H:%M:%S", // 25 12 2023 14:30:45
    "%d.%m.%Y %H.%M.%S", // 25.12.2023 14.30.45
    // Database/Log formats
    "%Y%m%d %H%M%S",         // 20231225 143045
    "%Y%m%d_%H%M%S",         // 20231225_143045
    "%Y%m%d-%H%M%S",         // 20231225-143045
    "%Y-%m-%d %H:%M:%S.%f",  // 2023-12-25 14:30:45.123456 (microseconds)
    "%Y-%m-%d %H:%M:%S.%3f", // 2023-12-25 14:30:45.123 (milliseconds)
    "%Y-%m-%d %H:%M:%S.%6f", // 2023-12-25 14:30:45.123456 (microseconds)
    // Alternative written formats
    "%B %d, %Y %H:%M:%S",    // December 25, 2023 14:30:45
    "%B %d, %Y %I:%M:%S %p", // December 25, 2023 2:30:45 PM
    "%d %B %Y %H:%M:%S",     // 25 December 2023 14:30:45
    "%b %d, %Y %H:%M:%S",    // Dec 25, 2023 14:30:45
    "%d %b %Y, %H:%M:%S",    // 25 Dec 2023, 14:30:45
    "%b %d, %Y %I:%M:%S %p", // Dec 25, 2023 2:30:45 PM
    "%d-%b-%Y %H:%M:%S",     // 25-Dec-2023 14:30:45
    "%d %b %Y %H:%M:%S",     // 25 Dec 2023 14:30:45
    // Year/month/day formats
    "%Y/%m/%d %H:%M:%S",    // 2023/12/25 14:30:45
    "%Y/%m/%d %I:%M:%S %p", // 2023/12/25 2:30:45 PM
    // Compact formats
    "%Y%m%d%H%M%S",  // 20231225143045
    "%Y%m%dT%H%M%S", // 20231225T143045
    // Additional common variants
    "%d.%m.%Y, %H:%M:%S", // 25.12.2023, 14:30:45
    "%d/%m/%Y, %H:%M:%S", // 25/12/2023, 14:30:45
    "%m/%d/%Y, %H:%M:%S", // 12/25/2023, 14:30:45
    "%Y-%m-%d, %H:%M:%S", // 2023-12-25, 14:30:45
    // Timestamps with different separators
    "%Y_%m_%d %H:%M:%S", // 2023_12_25 14:30:45
    "%Y.%m.%d %H:%M:%S", // 2023.12.25 14:30:45
    "%d_%m_%Y %H:%M:%S", // 25_12_2023 14:30:45
];

#[derive(Debug, thiserror::Error)]
#[error("Datetime not found")]
pub struct DateTimeNotFound;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DateTime(NaiveDateTime);

impl DateTime {
    pub fn into_inner(self) -> NaiveDateTime {
        self.0
    }
}

impl From<NaiveDateTime> for DateTime {
    fn from(value: NaiveDateTime) -> Self {
        DateTime(value)
    }
}

impl FromStr for DateTime {
    type Err = DateTimeNotFound;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let naive_dt = DATETIME_FORMATS
            .iter()
            .map(|fmt| chrono::NaiveDateTime::parse_from_str(s, fmt))
            .find_map(|result| result.ok())
            .ok_or(DateTimeNotFound)?;
        Ok(DateTime(naive_dt))
    }
}

impl Deref for DateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
