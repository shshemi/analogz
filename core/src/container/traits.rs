use regex::Regex;

use crate::container::ArcStr;

pub trait Find {
    fn find(&self, corpus: &ArcStr) -> Option<ArcStr>;
}

impl Find for &str {
    fn find(&self, corpus: &ArcStr) -> Option<ArcStr> {
        corpus.as_str().find(self).map(|start| {
            let end = start + self.len();
            corpus.slice(start..end)
        })
    }
}

impl Find for Regex {
    fn find(&self, corpus: &ArcStr) -> Option<ArcStr> {
        self.find(corpus.as_str())
            .map(|m| corpus.slice(m.start()..m.end()))
    }
}
