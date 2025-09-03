use std::ops::Deref;

use chrono::NaiveDateTime;
use dateparser::DateTimeUtc;

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

impl From<DateTimeUtc> for DateTime {
    fn from(value: DateTimeUtc) -> Self {
        DateTime(value.0.naive_utc())
    }
}

impl Deref for DateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
