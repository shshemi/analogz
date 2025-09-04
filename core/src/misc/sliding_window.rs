use std::iter::Skip;

use crate::{
    containers::ArcStr,
    misc::chars::{CharIndices, CharIndicesExt},
};

pub struct SlidingWindow {
    astr: ArcStr,
    start: CharIndices,
    end: Skip<CharIndices>,
}

impl Iterator for SlidingWindow {
    type Item = ArcStr;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start.next()?.0;
        let (end, c) = self.end.next()?;
        let slice = self.astr.slice(start..(end + c.len_utf8()));
        (!slice.is_empty()).then_some(slice)
    }
}

pub trait SlidingWindowExt {
    fn sliding_window(&self, size: usize) -> SlidingWindow;
}

impl SlidingWindowExt for ArcStr {
    fn sliding_window(&self, size: usize) -> SlidingWindow {
        if size == 0 {
            panic!("Invalid size: {size}")
        }

        SlidingWindow {
            astr: self.clone(),
            start: self.char_indices(),
            end: self.char_indices().skip(size.saturating_sub(1)),
        }
    }
}

#[cfg(test)]
mod sliding_window_tests {
    use super::{ArcStr, SlidingWindowExt};

    // Helper to collect windows into plain Strings (content only).
    fn collect_windows(s: &str, size: usize) -> Vec<String> {
        ArcStr::new(s)
            .sliding_window(size)
            .map(|w| w.as_str().to_string())
            .collect()
    }

    #[test]
    #[should_panic]
    fn size_zero_panics() {
        let _ = ArcStr::new("abc").sliding_window(0);
    }

    #[test]
    fn empty_input_yields_no_windows() {
        let out: Vec<_> = ArcStr::new("").sliding_window(1).collect();
        assert!(out.is_empty());
    }

    #[test]
    fn size_one_yields_each_char_as_singleton_window() {
        let out = collect_windows("abc", 1);
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn size_larger_than_char_count_yields_no_windows() {
        let out = collect_windows("abc", 4);
        assert!(out.is_empty());
    }

    #[test]
    fn windows_overlap_by_step_one() {
        // Length 4, size 3 -> 2 windows
        let out = collect_windows("abcd", 3);
        assert_eq!(out, vec!["abc", "bcd"]);
    }

    #[test]
    fn count_is_len_minus_size_plus_one_when_size_leq_len() {
        let s = "hello"; // 5 chars
        let out = collect_windows(s, 3);
        assert_eq!(out.len(), 5 - 3 + 1);
    }

    #[test]
    fn windows_have_exactly_size_characters() {
        let size = 3;
        let all_len_ok = ArcStr::new("abcde")
            .sliding_window(size)
            .all(|w| w.chars().count() == size);
        assert!(all_len_ok);
    }

    #[test]
    fn unicode_multibyte_characters_form_windows_by_scalar_count() {
        // "aé中🦀" -> 1,2,3,4-byte scalars; windows of size 2
        let out = collect_windows("aé中🦀", 2);
        assert_eq!(out, vec!["aé", "é中", "中🦀"]);
    }

    #[test]
    fn combining_mark_is_treated_as_its_own_scalar() {
        // 'a' + COMBINING ACUTE + 'b' -> windows of size 2
        let s = "a\u{0301}b";
        let out = collect_windows(s, 2);
        assert_eq!(out, vec!["a\u{0301}", "\u{0301}b"]);
    }

    #[test]
    fn fused_after_exhaustion() {
        let mut it = ArcStr::new("abc").sliding_window(2);
        assert_eq!(it.next().unwrap().as_str(), "ab");
        assert_eq!(it.next().unwrap().as_str(), "bc");
        assert!(it.next().is_none());
        assert!(it.next().is_none()); // still None after exhaustion
    }

    #[test]
    fn slice_input_produces_windows_relative_to_slice() {
        let base = ArcStr::new("hello world");
        let slice = base.slice(6..); // "world"
        let out: Vec<_> = slice
            .sliding_window(3)
            .map(|w| w.as_str().to_string())
            .collect();
        assert_eq!(out, vec!["wor", "orl", "rld"]);
    }

    #[test]
    fn every_window_is_non_empty() {
        let all_non_empty = ArcStr::new("abc")
            .sliding_window(2)
            .all(|w| !w.as_str().is_empty());
        assert!(all_non_empty);
    }

    #[test]
    fn windows_are_subslices_of_original_arcstr() {
        // Use `contains` to verify windows share the same backing Arc and lie within bounds.
        let base = ArcStr::new("abcdef");
        let base_clone = base.clone();
        let all_contained = base.sliding_window(3).all(|w| base_clone.contains(w));
        assert!(all_contained);
    }
}
