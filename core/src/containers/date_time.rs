use std::{ops::Deref, str::FromStr};

use chrono::NaiveDateTime;

pub const DATETIME_FORMATS: &[&str] = &[
    // Y-
    "%Y-%m-%dT%H:%M:%S%.fZ",
    "%Y-%m-%dT%H:%M:%S%:z",
    "%Y-%m-%dT%H:%M:%SZ",
    "%Y-%m-%dT%H:%M:%S%.3f",
    "%Y-%m-%dT%H:%M:%S%.6f",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%d %H:%M:%S",
    "%Y %b %d %H:%M:%S",
    "%Y_%m_%d %H:%M:%S",
    "%Y.%m.%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y%m%d %H%M%S",
    "%Y%m%d_%H%M%S",
    "%Y%m%d-%H%M%S",
    "%Y-%m-%d %H:%M:%S.%f",
    "%Y-%m-%d %H:%M:%S.%3f",
    "%Y-%m-%d %H:%M:%S.%6f",
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d %I:%M:%S %p",
    "%Y%m%d%H%M%S",
    "%Y%m%dT%H%M%S",
    "%Y-%m-%d, %H:%M:%S",
    // a-
    "%a, %d %b %Y %H:%M:%S",
    "%a %b %d %H:%M:%S %Y",
    // b-
    "%b %d %H:%M:%S %Y",
    "%b %d, %Y %H:%M:%S",
    "%b %d, %Y %I:%M:%S %p",
    // B-
    "%B %d, %Y %H:%M:%S",
    "%B %d, %Y %I:%M:%S %p",
    // d-
    "%d/%m/%Y %H:%M:%S",
    "%d %m %Y %H:%M:%S",
    "%d %b %Y %H:%M:%S",
    "%d %b %Y, %H:%M:%S",
    "%d-%b-%Y %H:%M:%S",
    "%d %B %Y %H:%M:%S",
    "%d.%m.%Y %H.%M.%S",
    "%d.%m.%Y %H:%M:%S",
    "%d-%m-%Y %H:%M:%S",
    "%d.%m.%Y, %H:%M:%S",
    "%d_%m_%Y %H:%M:%S",
    "%d/%m/%Y, %H:%M:%S",
    // m-
    "%m/%d/%Y %H:%M:%S",
    "%m/%d/%Y %I:%M:%S %p",
    "%m/%d/%Y %I:%M %p",
    "%m/%d/%Y, %H:%M:%S",
    "%m-%d-%Y %H:%M:%S",
    "%m-%d-%Y %I:%M:%S %p",
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
            .map(|fmt| chrono::NaiveDateTime::parse_and_remainder(s, fmt))
            .find_map(|result| result.ok())
            .ok_or(DateTimeNotFound)?;
        Ok(DateTime(naive_dt.0))
    }
}

impl Deref for DateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
