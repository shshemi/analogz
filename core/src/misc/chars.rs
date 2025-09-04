use crate::containers::ArcStr;

#[derive(Debug)]
pub struct CharsIndices {
    astr: ArcStr,
    offset: usize,
}

impl Iterator for CharsIndices {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.astr.as_str().chars().next().map(|c| {
            let o = self.offset;
            let l = c.len_utf8();
            self.offset += l;
            self.astr = self.astr.slice(l..);
            (o, c)
        })
    }
}

pub trait CharsExt {
    fn chars(self) -> CharsIndices;
}

impl CharsExt for ArcStr {
    fn chars(self) -> CharsIndices {
        CharsIndices {
            astr: self,
            offset: 0,
        }
    }
}

#[cfg(test)]
mod chars_indices_tests {
    use super::*;

    // Small helper to build the iterator from a &str
    fn mk(s: &str) -> CharsIndices {
        CharsIndices {
            astr: ArcStr::new(s),
            offset: 0,
        }
    }

    #[test]
    fn empty_string_returns_none() {
        let mut it = mk("");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn ascii_single_char_yields_zero_offset() {
        let mut it = mk("a");
        assert_eq!(it.next(), Some((0, 'a')));
    }

    #[test]
    fn ascii_sequence_offsets_are_incremental_by_one() {
        let mut it = mk("abc");
        assert_eq!(it.next(), Some((0, 'a')));
        assert_eq!(it.next(), Some((1, 'b')));
        assert_eq!(it.next(), Some((2, 'c')));
    }

    #[test]
    fn multibyte_2byte_char_correct_offset_advance() {
        // 'é' is 2 bytes in UTF-8
        let mut it = mk("é");
        assert_eq!(it.next(), Some((0, 'é')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn multibyte_3byte_char_correct_offset_advance() {
        // '€' is 3 bytes in UTF-8
        let mut it = mk("€");
        assert_eq!(it.next(), Some((0, '€')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn multibyte_4byte_char_correct_offset_advance() {
        // '🦀' is 4 bytes in UTF-8
        let mut it = mk("🦀");
        assert_eq!(it.next(), Some((0, '🦀')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mixed_ascii_and_multibyte_offsets_are_correct() {
        // "aé中🦀" -> byte offsets: a(0), é(1), 中(3), 🦀(6)
        let mut it = mk("aé中🦀");
        assert_eq!(it.next(), Some((0, 'a')));
        assert_eq!(it.next(), Some((1, 'é')));
        assert_eq!(it.next(), Some((3, '中')));
        assert_eq!(it.next(), Some((6, '🦀')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn iterator_is_fused_after_exhaustion() {
        let mut it = mk("a");
        assert_eq!(it.next(), Some((0, 'a')));
        assert_eq!(it.next(), None);
        // Calling next() again should still be None
        assert_eq!(it.next(), None);
    }

    #[test]
    fn offsets_are_strictly_increasing() {
        let mut it = mk("aé中🦀xyz");
        let offsets: Vec<usize> = std::iter::from_fn(|| it.next().map(|(i, _)| i)).collect();
        assert!(offsets.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn offsets_are_relative_to_provided_slice() {
        // Base: "hello world", slice: "lo world" (starts at byte 3)
        let base = ArcStr::new("hello world");
        let slice = base.slice(3..); // "lo world"
        let mut it = CharsIndices {
            astr: slice,
            offset: 0,
        };
        // Offsets should be relative to the slice (i.e., start at 0)
        assert_eq!(it.next(), Some((0, 'l')));
        assert_eq!(it.next(), Some((1, 'o')));
    }

    #[test]
    fn combining_mark_offsets_are_per_scalar_not_grapheme() {
        // "a\u{0301}" is 'a' + COMBINING ACUTE ACCENT
        let s = "a\u{0301}";
        let mut it = mk(s);
        assert_eq!(it.next(), Some((0, 'a')));
        // The combining mark starts at byte offset 1 in UTF-8
        assert_eq!(it.next(), Some((1, '\u{0301}')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn regional_indicator_flag_is_two_scalars_two_offsets() {
        // "🇫🇮" is two regional indicators: U+1F1EB and U+1F1EE (each 4 bytes)
        let mut it = mk("🇫🇮");
        assert_eq!(it.next(), Some((0, '🇫')));
        assert_eq!(it.next(), Some((4, '🇮')));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn original_arcstr_is_not_mutated_when_cloned_into_iterator() {
        let base = ArcStr::new("stay");
        let mut it = CharsIndices {
            astr: base.clone(),
            offset: 0,
        };
        // Consume iterator
        while it.next().is_some() {}
        // The original ArcStr remains unchanged
        assert_eq!(base.as_str(), "stay");
    }

    #[test]
    fn next_never_yields_empty_chars() {
        let mut it = mk("ok");
        // Each yielded char must have non-zero UTF-8 length
        for (_, ch) in it.by_ref() {
            assert!(ch.len_utf8() > 0);
        }
    }

    #[test]
    fn chars_on_empty_returns_none() {
        let mut it = ArcStr::new("").chars();
        assert_eq!(it.next(), None);
    }

    #[test]
    fn chars_starts_with_zero_offset_for_ascii() {
        let mut it = ArcStr::new("a").chars();
        assert_eq!(it.next(), Some((0, 'a')));
    }

    #[test]
    fn chars_starts_with_zero_offset_for_multibyte() {
        // '€' is 3 bytes; first yielded offset must be 0
        let mut it = ArcStr::new("€").chars();
        assert_eq!(it.next(), Some((0, '€')));
    }

    #[test]
    fn chars_on_slice_offsets_are_relative_to_slice() {
        let base = ArcStr::new("hello world");
        let slice = base.slice(6..); // "world"
        let mut it = slice.chars();
        assert_eq!(it.next(), Some((0, 'w')));
    }

    #[test]
    fn chars_produces_same_sequence_as_std_char_indices_for_whole_string() {
        let s = "aé中🦀";
        let ours: Vec<(usize, char)> = ArcStr::new(s).chars().collect();
        let std_ci: Vec<(usize, char)> = s.char_indices().collect();
        assert_eq!(ours, std_ci);
    }
}
