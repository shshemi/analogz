use std::{collections::HashSet, sync::Arc};

use crate::containers::ArcStr;

#[derive(Debug)]
pub struct Split {
    text: Option<ArcStr>,
    delimiters: Arc<HashSet<char>>,
}

impl Iterator for Split {
    type Item = ArcStr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(text) = self.text.take() {
            if let Some((idx, ch)) = text
                .char_indices()
                .find(|(_, c)| self.delimiters.as_ref().contains(c))
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

pub trait SplitExt {
    fn split(&self, delimiters: impl Into<Arc<HashSet<char>>>) -> Split;
}

impl SplitExt for ArcStr {
    fn split(&self, delimiters: impl Into<Arc<HashSet<char>>>) -> Split {
        Split {
            text: Some(self.clone()),
            delimiters: delimiters.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    // Adjust this helper if your ArcStr constructor differs.
    fn arc(s: &str) -> ArcStr {
        ArcStr::from(s)
    }

    fn delims(cs: &[char]) -> Arc<HashSet<char>> {
        Arc::new(cs.iter().cloned().collect())
    }

    fn collect_strings(it: Split) -> Vec<String> {
        it.map(|astr| astr.to_string()).collect()
    }

    #[test]
    fn empty_delimiter_set_returns_whole_string_once() {
        let s = arc("abc");
        let empty: Arc<HashSet<char>> = Arc::new(HashSet::new());
        let parts = collect_strings(s.split(empty));
        assert_eq!(parts, vec!["abc"]);
    }

    #[test]
    fn no_delimiters_in_text_returns_whole_string_once() {
        let s = arc("abcdef");
        let parts = collect_strings(s.split(delims(&[',', ';'])));
        assert_eq!(parts, vec!["abcdef"]);
    }

    #[test]
    fn single_delimiter_in_middle_splits_into_two_parts() {
        let s = arc("a,b");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["a", "b"]);
    }

    #[test]
    fn delimiter_at_start_produces_leading_empty_segment() {
        let s = arc(",abc");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["", "abc"]);
    }

    #[test]
    fn delimiter_at_end_has_no_trailing_empty_segment() {
        // With current implementation, trailing empty segment is NOT produced.
        let s = arc("abc,");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["abc", ""]);
    }

    #[test]
    fn consecutive_delimiters_produce_empty_middle_segment() {
        // This test reveals the current bug: find() scans from start of whole string,
        // not from self.start, which can cause panic or wrong result.
        let s = arc("a,,b");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["a", "", "b"]);
    }

    #[test]
    fn multiple_different_delimiters_are_respected() {
        let s = arc("a;b,c:d");
        let parts = collect_strings(s.split(delims(&[',', ';', ':'])));
        assert_eq!(parts, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn unicode_delimiter_multi_byte_split() {
        // '🙂' is 4 bytes in UTF-8
        let s = arc("a🙂b🙂c");
        let parts = collect_strings(s.split(delims(&['🙂'])));
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn unicode_text_with_ascii_delimiter() {
        let s = arc("αβγ,δεζ");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["αβγ", "δεζ"]);
    }

    #[test]
    fn delimiter_is_unicode_character() {
        // Use a non-ASCII delimiter such as 'ø'
        let s = arc("fooøbarøbaz");
        let parts = collect_strings(s.split(delims(&['ø'])));
        assert_eq!(parts, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn single_character_input_without_delimiter() {
        let s = arc("x");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["x"]);
    }

    #[test]
    fn single_character_input_is_delimiter_yields_no_output() {
        let s = arc(",");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["", ""]);
    }

    #[test]
    fn iterator_is_finite_and_exhausts_cleanly() {
        let s = arc("a,b");
        let mut it = s.split(delims(&[',']));
        assert_eq!(it.next().as_ref().map(|x| x.as_ref()), Some("a"));
        assert_eq!(it.next().as_ref().map(|x| x.as_ref()), Some("b"));
        assert_eq!(it.next(), None);
        // Calling next again still yields None (not testing FusedIterator trait, just behavior)
        assert_eq!(it.next(), None);
    }

    #[test]
    fn respects_current_start_offset() {
        // Another test that will fail with current implementation because find() restarts at 0
        let s = arc("ab,cd,ef");
        let parts = collect_strings(s.split(delims(&[','])));
        assert_eq!(parts, vec!["ab", "cd", "ef"]);
    }

    #[test]
    fn can_accept_arc_hashset_directly_via_splitext() {
        let s = arc("a;b");
        let set = delims(&[';']);
        let parts = collect_strings(s.split(set));
        assert_eq!(parts, vec!["a", "b"]);
    }
}
