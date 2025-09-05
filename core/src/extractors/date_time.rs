use dateparser::DateTimeUtc;

use crate::{
    containers::{ArcStr, DateTime},
    misc::{round_robin::IntoRoundRobin, sliding_window::SlidingWindowExt},
};

#[derive(Debug, Clone)]
pub struct DateTimeExtractor {
    min_len: usize,
    max_len: usize,
}

impl Default for DateTimeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DateTimeExtractor {
    pub fn new() -> Self {
        Self {
            min_len: 10,
            max_len: 42,
        }
    }

    pub fn extract(&self, text: ArcStr) -> Option<DateTime> {
        (self.min_len..self.max_len)
            .rev()
            .map(|size| text.sliding_window(size))
            .round_robin()
            .find_map(|win| win.parse::<DateTimeUtc>().ok().map(|dt| dt.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_various_datetime_formats() {
        let extractor = DateTimeExtractor::new();

        // ISO 8601 formats
        assert!(
            extractor.extract("2023-12-25T15:30:45Z".into()).is_some(),
            "ISO 8601 with Z"
        );
        assert!(
            extractor
                .extract("2023-12-25T15:30:45+00:00".into())
                .is_some(),
            "ISO 8601 with timezone offset"
        );
        assert!(
            extractor
                .extract("2023-12-25T15:30:45.123Z".into())
                .is_some(),
            "ISO 8601 with milliseconds"
        );
        assert!(
            extractor
                .extract("2023-12-25T15:30:45.123456Z".into())
                .is_some(),
            "ISO 8601 with microseconds"
        );
        assert!(
            extractor
                .extract("2023-12-25T15:30:45+05:30".into())
                .is_some(),
            "ISO 8601 with positive timezone"
        );
        assert!(
            extractor
                .extract("2023-12-25T15:30:45-08:00".into())
                .is_some(),
            "ISO 8601 with negative timezone"
        );

        // RFC 2822 / RFC 822 formats
        assert!(
            extractor
                .extract("Mon, 25 Dec 2023 15:30:45 GMT".into())
                .is_some(),
            "RFC 2822 format"
        );
        assert!(
            extractor
                .extract("Mon, 25 Dec 2023 15:30:45 +0000".into())
                .is_some(),
            "RFC 2822 with numeric timezone"
        );
        assert!(
            extractor
                .extract("25 Dec 2023 15:30:45 GMT".into())
                .is_some(),
            "RFC 2822 without day name"
        );

        // Common date formats
        assert!(
            extractor.extract("2023-12-25 15:30:45".into()).is_some(),
            "YYYY-MM-DD HH:MM:SS"
        );
        assert!(
            extractor.extract("2023/12/25 15:30:45".into()).is_some(),
            "YYYY/MM/DD HH:MM:SS"
        );
        assert!(
            extractor.extract("25/12/2023 15:30:45".into()).is_some(),
            "DD/MM/YYYY HH:MM:SS"
        );
        assert!(
            extractor.extract("12/25/2023 15:30:45".into()).is_some(),
            "MM/DD/YYYY HH:MM:SS"
        );
        assert!(
            extractor.extract("25-12-2023 15:30:45".into()).is_some(),
            "DD-MM-YYYY HH:MM:SS"
        );
        assert!(
            extractor.extract("12-25-2023 15:30:45".into()).is_some(),
            "MM-DD-YYYY HH:MM:SS"
        );

        // Date only formats
        assert!(
            extractor.extract("2023-12-25".into()).is_some(),
            "YYYY-MM-DD date only"
        );
        assert!(
            extractor.extract("2023/12/25".into()).is_some(),
            "YYYY/MM/DD date only"
        );
        assert!(
            extractor.extract("25/12/2023".into()).is_some(),
            "DD/MM/YYYY date only"
        );
        assert!(
            extractor.extract("12/25/2023".into()).is_some(),
            "MM/DD/YYYY date only"
        );
        assert!(
            extractor.extract("25-12-2023".into()).is_some(),
            "DD-MM-YYYY date only"
        );
        assert!(
            extractor.extract("12-25-2023".into()).is_some(),
            "MM-DD-YYYY date only"
        );

        // Time with AM/PM
        assert!(
            extractor.extract("2023-12-25 3:30:45 PM".into()).is_some(),
            "12-hour format with PM"
        );
        assert!(
            extractor.extract("2023-12-25 11:30:45 AM".into()).is_some(),
            "12-hour format with AM"
        );
        assert!(
            extractor.extract("25/12/2023 3:30:45 PM".into()).is_some(),
            "DD/MM/YYYY with PM"
        );
        assert!(
            extractor.extract("12/25/2023 11:30:45 AM".into()).is_some(),
            "MM/DD/YYYY with AM"
        );

        // Named months
        assert!(
            extractor.extract("25 Dec 2023 15:30:45".into()).is_some(),
            "DD MMM YYYY HH:MM:SS"
        );
        assert!(
            extractor.extract("Dec 25, 2023 15:30:45".into()).is_some(),
            "MMM DD, YYYY HH:MM:SS"
        );
        assert!(
            extractor
                .extract("December 25, 2023 15:30:45".into())
                .is_some(),
            "MMMM DD, YYYY HH:MM:SS"
        );
        assert!(
            extractor
                .extract("25 December 2023 15:30:45".into())
                .is_some(),
            "DD MMMM YYYY HH:MM:SS"
        );
        assert!(
            extractor.extract("2023 Dec 25 15:30:45".into()).is_some(),
            "YYYY MMM DD HH:MM:SS"
        );

        // Unix timestamp formats
        assert!(
            extractor.extract("1703516245".into()).is_some(),
            "Unix timestamp (10 digits)"
        );
        assert!(
            extractor.extract("1703516245123".into()).is_some(),
            "Unix timestamp with milliseconds (13 digits)"
        );

        // Different separators and formats
        assert!(
            extractor.extract("2023.12.25 15:30:45".into()).is_some(),
            "Dot separated date"
        );
        assert!(
            extractor.extract("2023 12 25 15:30:45".into()).is_some(),
            "Space separated date"
        );
        assert!(
            extractor.extract("25.12.2023 15:30:45".into()).is_some(),
            "DD.MM.YYYY format"
        );
        assert!(
            extractor.extract("25 12 2023 15:30:45".into()).is_some(),
            "DD MM YYYY format"
        );

        // With weekdays
        assert!(
            extractor
                .extract("Monday, 25 Dec 2023 15:30:45".into())
                .is_some(),
            "With full weekday name"
        );
        assert!(
            extractor
                .extract("Mon, 25 Dec 2023 15:30:45".into())
                .is_some(),
            "With abbreviated weekday"
        );

        // Different time formats
        assert!(
            extractor.extract("2023-12-25 15:30".into()).is_some(),
            "Without seconds"
        );
        assert!(
            extractor.extract("2023-12-25 3:30 PM".into()).is_some(),
            "12-hour without seconds"
        );
        assert!(
            extractor.extract("2023-12-25T15:30".into()).is_some(),
            "ISO format without seconds"
        );

        // Text with embedded dates
        assert!(
            extractor
                .extract("The meeting is on 2023-12-25 at 15:30:45".into())
                .is_some(),
            "Date embedded in text"
        );
        assert!(
            extractor
                .extract("Event: Dec 25, 2023 3:30 PM - Don't miss it!".into())
                .is_some(),
            "Date in descriptive text"
        );
        assert!(
            extractor
                .extract("Deadline 25/12/2023 23:59:59 sharp".into())
                .is_some(),
            "Date at end of text"
        );
        assert!(
            extractor
                .extract("From 2023-01-01T00:00:00Z to 2023-12-31T23:59:59Z".into())
                .is_some(),
            "Multiple dates (should find first)"
        );
    }

    #[test]
    fn test_invalid_formats_should_fail() {
        let extractor = DateTimeExtractor::new();

        // Edge cases and potentially invalid formats
        assert!(
            extractor.extract("not a date".into()).is_none(),
            "Plain text"
        );
        assert!(extractor.extract("".into()).is_none(), "Empty string");
        assert!(extractor.extract("2023".into()).is_none(), "Year only");
        assert!(extractor.extract("12".into()).is_none(), "Too short");
        assert!(
            extractor.extract("2023-13-25".into()).is_none(),
            "Invalid month"
        );
        assert!(
            extractor.extract("2023-12-32".into()).is_none(),
            "Invalid day"
        );
        assert!(
            extractor.extract("2023-12-25 25:30:45".into()).is_none(),
            "Invalid hour"
        );
        assert!(
            extractor.extract("2023-12-25 15:60:45".into()).is_none(),
            "Invalid minute"
        );
        assert!(
            extractor.extract("2023-12-25 15:30:60".into()).is_none(),
            "Invalid second"
        );
    }

    #[test]
    fn test_extractor_default() {
        let extractor1 = DateTimeExtractor::new();
        let extractor2 = DateTimeExtractor::default();

        assert_eq!(extractor1.min_len, extractor2.min_len);
        assert_eq!(extractor1.max_len, extractor2.max_len);
    }

    #[test]
    fn test_boundary_conditions() {
        let extractor = DateTimeExtractor::new();

        // Test minimum length boundary (should be 10 based on your constructor)
        let short_date = "2023-12-25"; // exactly 10 characters
        assert!(extractor.extract(short_date.into()).is_some());

        // Test very short string (below min_len)
        let too_short = "2023-12"; // 7 characters
        assert!(extractor.extract(too_short.into()).is_none());

        // Test long valid datetime string
        let long_date = "Monday, December 25, 2023 15:30:45.123456 GMT";
        let result = extractor.extract(long_date.into());
        // This might or might not work depending on the max_len and parsing capability
        println!("Long date result: {:?}", result);
    }

    #[test]
    fn test_various_timezones() {
        let extractor = DateTimeExtractor::new();

        let timezone_formats = vec![
            "2023-12-25T15:30:45Z",
            "2023-12-25T15:30:45+00:00",
            "2023-12-25T15:30:45-05:00",
            "2023-12-25T15:30:45+09:30",
            "2023-12-25T15:30:45 UTC",
            "2023-12-25T15:30:45 GMT",
            "2023-12-25T15:30:45 EST",
            "2023-12-25T15:30:45 PST",
        ];

        for format in timezone_formats {
            let result = extractor.extract(format.into());
            println!("Timezone format '{}': {:?}", format, result.is_some());
        }
    }

    #[test]
    fn test_sliding_window_behavior() {
        let extractor = DateTimeExtractor::new();

        // Test string where the date is not at the beginning
        let text_with_date = "The important date is 2023-12-25T15:30:45Z for the event";
        let result = extractor.extract(text_with_date.into());
        assert!(result.is_some(), "Should find date in the middle of text");

        // Test string where multiple dates exist (should find the first one due to round-robin)
        let multi_date_text = "Start: 2023-01-01T00:00:00Z End: 2023-12-31T23:59:59Z";
        let result = extractor.extract(multi_date_text.into());
        assert!(result.is_some(), "Should find at least one date");
    }
}
