use regex::Regex;

use crate::containers::ArcStr;

pub trait Pattern {
    type Searcher: Searcher;

    fn into_searcher(self, astr: ArcStr) -> Self::Searcher;
}

pub trait Searcher {
    fn next_match(&mut self) -> Option<(usize, usize)>;
}

impl<'a> Pattern for &'a str {
    type Searcher = StrSearcher<'a>;

    fn into_searcher(self, astr: ArcStr) -> Self::Searcher {
        Self::Searcher {
            astr,
            pat: self,
            offset: 0,
        }
    }
}

impl Pattern for Regex {
    type Searcher = RegexSearcher;

    fn into_searcher(self, astr: ArcStr) -> Self::Searcher {
        Self::Searcher { astr, pat: self }
    }
}

pub struct StrSearcher<'a> {
    astr: ArcStr,
    pat: &'a str,
    offset: usize,
}

impl<'a> StrSearcher<'a> {
    pub fn new(astr: ArcStr, pat: &'a str) -> Self {
        Self {
            astr,
            pat,
            offset: 0,
        }
    }
}

impl<'a> Searcher for StrSearcher<'a> {
    fn next_match(&mut self) -> Option<(usize, usize)> {
        if let Some(start) = self.astr.as_str()[self.offset..].find(self.pat) {
            let (_, n) = self.astr.split_at(start);
            self.astr = n;
            Some((start, start + self.pat.len()))
        } else {
            None
        }
    }
}

pub struct RegexSearcher {
    astr: ArcStr,
    pat: Regex,
}

impl RegexSearcher {
    pub fn new(astr: ArcStr, pat: Regex) -> Self {
        Self { astr, pat }
    }
}

impl Searcher for RegexSearcher {
    fn next_match(&mut self) -> Option<(usize, usize)> {
        if let Some((start, end)) = self
            .pat
            .find(self.astr.as_str())
            .map(|m| (m.start(), m.end()))
        {
            let (_, n) = self.astr.split_at(start);
            self.astr = n;
            Some((start, end))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::containers::ArcStr;

    #[test]
    fn match_at_start() {
        let astr = ArcStr::from("abc");
        let mut searcher = StrSearcher::new(astr, "a");
        assert_eq!(searcher.next_match(), Some((0, 1)));
    }

    #[test]
    fn match_in_middle() {
        let mut searcher = StrSearcher::new(ArcStr::from("xabc"), "ab");
        assert_eq!(searcher.next_match(), Some((1, 3)));
    }

    #[test]
    fn no_match_returns_none() {
        let mut searcher = StrSearcher::new(ArcStr::from("abc"), "d");
        assert_eq!(searcher.next_match(), None);
    }

    #[test]
    fn pattern_equals_entire_string() {
        let mut searcher = StrSearcher::new(ArcStr::from("abc"), "abc");
        assert_eq!(searcher.next_match(), Some((0, 3)));
    }

    #[test]
    fn pattern_longer_than_string_returns_none() {
        let mut searcher = StrSearcher::new(ArcStr::from("ab"), "abcd");
        assert_eq!(searcher.next_match(), None);
    }

    #[test]
    fn empty_string_and_empty_pattern() {
        let mut searcher = StrSearcher::new(ArcStr::from(""), "");
        // Rust's str::find returns 0 for an empty needle in an empty haystack
        assert_eq!(searcher.next_match(), Some((0, 0)));
    }

    #[test]
    fn empty_pattern_initial_return() {
        let mut searcher = StrSearcher::new(ArcStr::from("abc"), "");
        assert_eq!(searcher.next_match(), Some((0, 0)));
    }

    #[test]
    fn empty_pattern_repeated_behavior() {
        let mut searcher = StrSearcher::new(ArcStr::from("xyz"), "");
        // Calling repeatedly should continue to find the empty pattern at position 0
        assert_eq!(searcher.next_match(), Some((0, 0)));
        assert_eq!(searcher.next_match(), Some((0, 0)));
    }

    #[test]
    fn progression_after_nonzero_start_shifts_internal_arcstr() {
        // First match is at a non-zero index; internal ArcStr is replaced with the suffix
        let mut searcher = StrSearcher::new(ArcStr::from("xxabc"), "abc");
        assert_eq!(searcher.next_match(), Some((2, 5)));
        // After the first call the internal ArcStr begins at the previous match start,
        // so the next match is found at index 0 relative to the new ArcStr.
        assert_eq!(searcher.next_match(), Some((0, 3)));
    }

    #[test]
    fn multiple_matches() {
        // There are two occurrences of "abc" in the string.
        let mut searcher = StrSearcher::new(ArcStr::from("xxabcxxabc"), "abc");
        // First call finds the first occurrence at a non-zero index.
        assert_eq!(searcher.next_match(), Some((2, 5)));
        // After the internal ArcStr is adjusted, the next call finds the pattern at index 0
        // relative to the updated ArcStr.
        assert_eq!(searcher.next_match(), Some((7, 3)));
    }
}
