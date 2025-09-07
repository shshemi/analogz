use crate::containers::ArcStr;

#[derive(Debug)]
pub struct Split<S> {
    text: Option<ArcStr>,
    split_chars: S,
}

impl<SP> Iterator for Split<SP>
where
    SP: SplitChars,
{
    type Item = ArcStr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(text) = self.text.take() {
            if let Some((idx, ch)) = text
                .char_indices()
                .find(|(_, c)| self.split_chars.contains(c))
            {
                let (next, _, text) = text.split_at_two(idx, idx + ch.len_utf8());
                self.text = Some(text);
                Some(next)
            } else {
                Some(text)
            }
        } else {
            None
        }
    }
}

pub trait SplitExt<S> {
    fn split(&self, pattern: S) -> Split<S>;
}

impl<S> SplitExt<S> for ArcStr
where
    S: SplitChars,
{
    fn split(&self, pattern: S) -> Split<S> {
        Split {
            text: Some(self.clone()),
            split_chars: pattern,
        }
    }
}

pub trait SplitChars {
    fn contains(&self, c: &char) -> bool;
}

impl SplitChars for &str {
    fn contains(&self, c: &char) -> bool {
        self.chars().any(|a| &a == c)
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
        assert_eq!(it.next().as_deref(), Some("a"));
        assert_eq!(it.next().as_deref(), Some("b"));
        assert_eq!(it.next(), None);
        // Call next() multiple times; still None
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn split_yields_entire_rest_when_no_more_delims() {
        // Ensures branch that yields remaining text when find() returns None
        let mut it = arc("a,bcd").split(",");
        assert_eq!(it.next().as_deref(), Some("a"));
        assert_eq!(it.next().as_deref(), Some("bcd"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn split_handles_long_input_without_delims() {
        let long = "x".repeat(10_000);
        assert_eq!(parts(&long, ","), vec![long]);
    }
}
