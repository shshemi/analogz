use chrono::NaiveDateTime;
use regex::Regex;

use super::{Extract, Location};

pub struct Datetime {
    inner: NaiveDateTime,
}

impl Datetime {
    pub fn inner(&self) -> &NaiveDateTime {
        &self.inner
    }

    pub fn into_inner(self) -> NaiveDateTime {
        self.inner
    }
}

pub struct Extractor {
    pats: Vec<DatePattern>,
}

impl Default for Extractor {
    fn default() -> Self {
        Extractor {
            pats: vec![
                // TODO: add more patterns
                DatePattern::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}", "%Y-%m-%d %H:%M:%S"),
            ],
        }
    }
}

impl Extract for Extractor {
    type Value = Datetime;

    fn extract(&self, slice: &str) -> Option<(super::Location, Self::Value)> {
        self.pats.iter().find_map(|pat| pat.extract(slice))
    }
}

struct DatePattern {
    re: Regex,
    fmt: &'static str,
}

impl DatePattern {
    fn new(re: &'static str, format: &'static str) -> Self {
        Self {
            re: Regex::new(re).unwrap(),
            fmt: format,
        }
    }

    fn extract(&self, corpus: &str) -> Option<(Location, Datetime)> {
        let c = self.re.find(corpus).map(|m| {
            let start = m.start();
            let end = m.end();
            (
                Location { start, end },
                Datetime {
                    inner: NaiveDateTime::parse_from_str(&corpus[start..end], self.fmt).unwrap(),
                },
            )
        });
        c
    }
}
