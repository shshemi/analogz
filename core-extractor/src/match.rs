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
