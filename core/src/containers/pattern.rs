use regex::Regex;

use crate::containers::ArcStr;

pub trait Pattern {
    type Searcher: Searcher;

    fn into_searcher(self, astr: ArcStr) -> Self::Searcher;
}

pub trait Searcher {
    fn next_match(&mut self) -> Option<(usize, usize)>;
}

pub struct StrSearcher<'a> {
    astr: ArcStr,
    pat: &'a str,
    offset: usize,
}

impl<'a> Searcher for StrSearcher<'a> {
    fn next_match(&mut self) -> Option<(usize, usize)> {
        if !self.pat.is_empty()
            && let Some(start) = self.astr.as_str()[self.offset..].find(self.pat)
        {
            let start = self.offset + start;
            let end = start + self.pat.len();
            self.offset = end;
            Some((start, end))
        } else {
            None
        }
    }
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

impl Pattern for Regex {
    type Searcher = RegexSearcher;

    fn into_searcher(self, astr: ArcStr) -> Self::Searcher {
        Self::Searcher { astr, pat: self }
    }
}
