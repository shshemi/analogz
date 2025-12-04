pub mod date_time;
pub mod ip_addr;
pub mod socket_addr;
use itertools::Itertools;

use crate::{containers::DateTime, misc::token_borders::TokenBorders};

#[derive(Debug)]
pub struct Match<T> {
    start: usize,
    end: usize,
    value: T,
}

impl<T> Match<T> {
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn into_value(self) -> T {
        self.value
    }
}

pub fn date_time(haystack: &str) -> impl Iterator<Item = Match<DateTime>> {
    std::iter::once(0)
        .chain(haystack.char_indices().filter_map(|(i, c)| {
            (c.is_ascii_whitespace() || c.is_ascii_punctuation()).then_some(i + 1)
        }))
        .map(|start| (start, &haystack[start..]))
        .filter_map(|(start, slice)| {
            let (value, rem) = DateTime::parse_and_remainder(slice).ok()?;
            let end = rem.as_ptr() as usize - haystack.as_ptr() as usize;
            Some(Match { start, end, value })
        })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn delete() {
        let txt = "  2020-01-02T03:04:05Z";
        let d = "  2020-01-02T03:04:05Z alskdjflkasjf";
        // DateTime::from_str(d).unwrap();
        DateTime::parse_and_remainder(d).unwrap();
    }

    #[test]
    fn empty_haystack_yields_no_matches() {
        let haystack = "";
        let matches: Vec<_> = date_time(haystack).collect();
        assert!(matches.is_empty(), "expected no matches for empty haystack");
    }

    #[test]
    fn finds_single_rfc3339_datetime() {
        let haystack = "prefix 2020-01-02T03:04:05Z suffix";
        let mut iter = date_time(haystack);
        let m = iter.next().expect("expected one match");
        assert!(iter.next().is_none(), "expected exactly one match");
        let slice = &haystack[m.start()..m.end()];
        assert_eq!(slice, "2020-01-02T03:04:05Z");
        assert!(m.end() > m.start());
        // consume the value to ensure Into works
        let _dt = m.into_value();
    }

    #[test]
    fn finds_multiple_datetimes_and_preserves_order() {
        let haystack = "first 2020-01-02T03:04:05Z middle 1999-12-31T23:59:59+01:00 end";
        let matches: Vec<_> = date_time(haystack).collect();
        assert_eq!(matches.len(), 2, "expected two datetime matches");
        let slices: Vec<&str> = matches
            .iter()
            .map(|m| &haystack[m.start()..m.end()])
            .collect();
        assert_eq!(slices[0], "2020-01-02T03:04:05Z");
        assert_eq!(slices[1], "1999-12-31T23:59:59+01:00");
        // ensure start positions are strictly increasing
        assert!(matches[0].start() < matches[1].start());
    }

    #[test]
    fn adjacent_datetimes_are_both_detected() {
        let haystack = "2020-01-02T03:04:05Z 2020-01-02T04:05:06Z";
        let matches: Vec<_> = date_time(haystack).collect();
        assert_eq!(matches.len(), 2, "expected two adjacent datetime matches");
        let slices: Vec<&str> = matches
            .iter()
            .map(|m| &haystack[m.start()..m.end()])
            .collect();
        assert_eq!(slices, vec!["2020-01-02T03:04:05Z", "2020-01-02T04:05:06Z"]);
    }

    #[test]
    fn repeated_same_datetime_matches_each_occurrence() {
        let haystack = "date:2020-01-02T03:04:05Z,2020-01-02T03:04:05Zx";
        let matches: Vec<_> = date_time(haystack).collect();
        assert_eq!(
            matches.len(),
            2,
            "expected two occurrences of the same datetime"
        );
        let slices: Vec<&str> = matches
            .iter()
            .map(|m| &haystack[m.start()..m.end()])
            .collect();
        assert_eq!(slices, vec!["2020-01-02T03:04:05Z", "2020-01-02T03:04:05Z"]);
    }
}
