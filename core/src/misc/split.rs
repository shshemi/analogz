use crate::containers::{ArcStr, Pattern, Searcher};

#[derive(Debug)]
pub struct Split<S> {
    astr: ArcStr,
    ser: S,
    start: usize,
}

impl<S> Split<S> {
    pub fn new<P>(astr: ArcStr, pat: P) -> Self
    where
        P: Pattern<Searcher = S>,
    {
        Self {
            astr: astr.clone(),
            ser: pat.into_searcher(astr),
            start: 0,
        }
    }
}

impl<S> Iterator for Split<S>
where
    S: Searcher,
{
    type Item = ArcStr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, end)) = self.ser.next_match() {
            let next = self.astr.slice(self.start..start);
            self.start = end;
            Some(next)
        } else {
            let len = self.astr.len();
            if self.start < len {
                let next = self.astr.slice(self.start..len);
                self.start = len;
                Some(next)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arc(s: &str) -> ArcStr {
        ArcStr::from(s)
    }

    fn parts(input: &str, pat: &str) -> Vec<String> {
        arc(input)
            .split(pat)
            .map(|s| s.as_ref().to_string())
            .collect()
    }

    #[test]
    fn split_empty_input_yields_single_empty() {
        assert_eq!(parts("", ","), vec![""]);
    }

    #[test]
    fn split_no_delimiter_returns_whole() {
        assert_eq!(parts("abc", ","), vec!["abc"]);
    }

    #[test]
    fn split_empty_pattern_never_matches() {
        // &str::chars() is empty → contains() is false → no split
        assert_eq!(parts("a,b,c", ""), vec!["a,b,c"]);
    }

    #[test]
    fn split_single_char_delim_basic() {
        assert_eq!(parts("a,b,c", ","), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_multi_char_set_any_matches() {
        assert_eq!(parts("a;b,c|d", ",;|"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn split_leading_delimiter_yields_leading_empty_field() {
        assert_eq!(parts(",a,b", ","), vec!["", "a", "b"]);
    }

    #[test]
    fn split_trailing_delimiter_yields_trailing_empty_field() {
        assert_eq!(parts("a,b,", ","), vec!["a", "b", ""]);
    }

    #[test]
    fn split_consecutive_delimiters_yield_empty_fields_between() {
        assert_eq!(parts("a,,b", ","), vec!["a", "", "b"]);
    }

    #[test]
    fn split_all_delimiters_only_yields_n_plus_one_empties() {
        assert_eq!(parts(",,,", ","), vec!["", "", "", ""]);
    }

    #[test]
    fn split_unicode_text_ascii_delim() {
        assert_eq!(parts("α,β,γ", ","), vec!["α", "β", "γ"]);
    }

    #[test]
    fn split_unicode_delimiter_emoji() {
        assert_eq!(parts("a😀b😀c", "😀"), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_unicode_delimiter_multibyte_letter() {
        assert_eq!(parts("fooøbarøbaz", "ø"), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn split_does_not_consume_past_end_after_none() {
        // After iterator returns None, it must keep returning None on subsequent calls.
        let mut it = arc("a,b").split(",");
        assert_eq!(it.next().unwrap(), "a");
        assert_eq!(it.next().unwrap(), "b");
        assert!(it.next().is_none());
        // Call next() multiple times; still None
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn split_yields_entire_rest_when_no_more_delims() {
        // Ensures branch that yields remaining text when find() returns None
        let mut it = arc("a,bcd").split(",");
        assert_eq!(it.next().unwrap(), "a");
        assert_eq!(it.next().unwrap(), "bcd");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn split_handles_long_input_without_delims() {
        let long = "x".repeat(10_000);
        assert_eq!(parts(&long, ","), vec![long]);
    }
}
