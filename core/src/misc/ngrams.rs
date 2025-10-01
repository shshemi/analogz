use crate::containers::{ArcStr, CutIndex};

pub struct NGrams {
    astr: ArcStr,
    cuts: CutIndex,
    i: usize,
    j: usize,
}

impl Iterator for NGrams {
    type Item = ArcStr;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.cuts.start(self.i)?;
        let end = self.cuts.end(self.j)?;
        if self.j < self.cuts.len() - 1 {
            self.j += 1;
        } else {
            self.i += 1;
            self.j = self.i;
        }
        Some(self.astr.slice(start..end))
    }
}

pub trait NGramsExt {
    fn ngrams(&self, split_chars: &[u8]) -> NGrams;
}

impl NGramsExt for ArcStr {
    fn ngrams(&self, split_chars: &[u8]) -> NGrams {
        let split_chars = split_chars.to_owned();
        NGrams {
            astr: self.clone(),
            cuts: CutIndex::build(self.as_bytes(), move |c| split_chars.contains(c)),
            i: 0,
            j: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::containers::ArcStr;

    // Helper function to collect all ngrams into a vector of strings for easier testing
    fn collect_ngrams(text: &str, split_chars: &str) -> Vec<String> {
        ArcStr::from(text)
            .ngrams(split_chars.as_bytes())
            .map(|s| s.to_string())
            .collect()
    }

    // Tests for normal cases
    #[test]
    fn test_simple_space_splitting() {
        let result = collect_ngrams("hello world test", " ");
        let expected = vec![
            "hello",
            "hello world",
            "hello world test",
            "world",
            "world test",
            "test",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_word() {
        let result = collect_ngrams("hello", " ");
        let expected = vec!["hello"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_two_words() {
        let result = collect_ngrams("hello world", " ");
        let expected = vec!["hello", "hello world", "world"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_three_words() {
        let result = collect_ngrams("one two three", " ");
        let expected = vec![
            "one",
            "one two",
            "one two three",
            "two",
            "two three",
            "three",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_punctuation_splitting() {
        let result = collect_ngrams("hello.world!test", ".!");
        let expected = vec![
            "hello",
            "hello.world",
            "hello.world!test",
            "world",
            "world!test",
            "test",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_unicode_text() {
        let result = collect_ngrams("café müller", " ");
        let expected = vec!["café", "café müller", "müller"];
        assert_eq!(result, expected);
    }

    // Edge case tests
    #[test]
    fn test_empty_string() {
        let result = collect_ngrams("", " ");
        let expected = vec![""];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_split_chars() {
        let result = collect_ngrams("hello world", "");
        // With no split chars, the entire string should be treated as one segment
        let expected = vec!["hello world"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_only_split_characters() {
        let result = collect_ngrams("   ", " ");
        let expected = vec!["", " ", "  ", "   ", "", " ", "  ", "", " ", ""];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_consecutive_split_characters() {
        let result = collect_ngrams("hello  world", " ");
        let expected = vec!["hello", "hello ", "hello  world", "", " world", "world"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_leading_split_characters() {
        let result = collect_ngrams(" hello world", " ");
        let expected = vec![
            "",
            " hello",
            " hello world",
            "hello",
            "hello world",
            "world",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trailing_split_characters() {
        let result = collect_ngrams("hello world ", " ");
        let expected = vec![
            "hello",
            "hello world",
            "hello world ",
            "world",
            "world ",
            "",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_leading_and_trailing_split_characters() {
        let result = collect_ngrams(" hello world ", " ");
        let expected = vec![
            "",
            " hello",
            " hello world",
            " hello world ",
            "hello",
            "hello world",
            "hello world ",
            "world",
            "world ",
            "",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_split_character_not_present() {
        let result = collect_ngrams("hello world", ",");
        let expected = vec!["hello world"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_all_characters_are_split_chars() {
        let result = collect_ngrams("hello", "helo");
        let expected = vec![
            "", "h", "he", "hel", "hell", "hello", "", "e", "el", "ell", "ello", "", "l", "ll",
            "llo", "", "l", "lo", "", "o", "",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_character_string() {
        let result = collect_ngrams("a", " ");
        let expected = vec!["a"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_split_character_string() {
        let result = collect_ngrams(" ", " ");
        let expected = vec!["", " ", ""];
        assert_eq!(result, expected);
    }

    // Iterator behavior tests
    #[test]
    fn test_iterator_can_be_collected_multiple_times() {
        let astr = ArcStr::from("hello world test");
        let ngrams1: Vec<String> = astr.ngrams(b" ").map(|s| s.to_string()).collect();
        let ngrams2: Vec<String> = astr.ngrams(b" ").map(|s| s.to_string()).collect();
        assert_eq!(ngrams1, ngrams2);
    }

    #[test]
    fn test_iterator_next_returns_none_when_exhausted() {
        let mut ngrams = ArcStr::from("hello world").ngrams(b" ");

        // Consume all items
        let items: Vec<_> = ngrams.by_ref().collect();
        assert_eq!(items.len(), 3); // "hello", "hello world", "world"

        // Further calls to next should return None
        assert_eq!(ngrams.next(), None);
        assert_eq!(ngrams.next(), None);
    }

    #[test]
    fn test_iterator_with_single_element() {
        let mut ngrams = ArcStr::from("hello").ngrams(b" ");
        assert_eq!(
            ngrams.next().map(|s| s.to_string()),
            Some("hello".to_string())
        );
        assert_eq!(ngrams.next(), None);
    }

    #[test]
    fn test_iterator_empty_case() {
        let mut ngrams = ArcStr::from("").ngrams(b" ");
        assert_eq!(ngrams.next().unwrap().as_ref(), "");
    }

    // Tests for specific ordering and completeness

    #[test]
    fn test_ngram_ordering_is_correct() {
        let result = collect_ngrams("a b c d", " ");
        let expected = vec![
            // Starting with first token
            "a", "a b", "a b c", "a b c d", // Starting with second token
            "b", "b c", "b c d", // Starting with third token
            "c", "c d", // Starting with fourth token
            "d",
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_ngram_count_formula() {
        // For n tokens, there should be n*(n+1)/2 ngrams
        let tokens = ["a", "b", "c", "d", "e"];
        let input = tokens.join(" ");
        let result = collect_ngrams(&input, " ");
        let n = tokens.len();
        let expected_count = n * (n + 1) / 2;
        assert_eq!(result.len(), expected_count);
    }

    // Performance edge cases

    #[test]
    fn test_repeated_characters() {
        let result = collect_ngrams("aaaa bbbb", " ");
        let expected = vec!["aaaa", "aaaa bbbb", "bbbb"];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alternating_pattern() {
        let result = collect_ngrams("a b a b", " ");
        let expected = vec![
            "a", "a b", "a b a", "a b a b", "b", "b a", "b a b", "a", "a b", "b",
        ];
        assert_eq!(result, expected);
    }
}
