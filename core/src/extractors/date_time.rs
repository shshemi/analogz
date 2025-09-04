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
            min_len: 11,
            max_len: 32,
        }
    }

    pub fn extract(&self, text: ArcStr) -> Option<(ArcStr, DateTime)> {
        (self.min_len..self.max_len)
            .map(|size| text.sliding_window(size))
            .round_robin()
            .find_map(|win| win.parse::<DateTimeUtc>().ok().map(|dt| (win, dt.into())))
    }
}
