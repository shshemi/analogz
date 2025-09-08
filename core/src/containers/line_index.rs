use itertools::Itertools;

use std::ops::Range;

use crate::containers::ArcSlice;

#[derive(Debug, Clone)]
pub struct LineIndex {
    line_ends: ArcSlice<usize>,
}

impl LineIndex {
    pub fn build(corpus: impl AsRef<str>) -> Self {
        let corpus = corpus.as_ref();
        if corpus.is_empty() {
            LineIndex {
                line_ends: Vec::default().into(),
            }
        } else {
            let line_ends = std::thread::scope(|scope| {
                let chunk_size = num_cpus::get();
                std::iter::once(0)
                    .chain(
                        corpus
                            .as_bytes()
                            .chunks(chunk_size)
                            .enumerate()
                            .map(move |(idx, slice)| (idx * chunk_size, slice))
                            .map(|(offset, slice)| {
                                scope.spawn(move || {
                                    slice.iter().enumerate().filter_map(move |(i, c)| {
                                        (*c == b'\n').then_some(offset + i)
                                    })
                                })
                            })
                            .flat_map(|hndl| hndl.join().unwrap()),
                    )
                    .chain([corpus.len()])
                    .collect_vec()
            });

            LineIndex {
                line_ends: line_ends.into(),
            }
        }
    }

    pub fn slice(&self, rng: Range<usize>) -> Self {
        LineIndex {
            line_ends: self.line_ends.slice(rng),
        }
    }

    pub fn line_start(&self, idx: usize) -> Option<usize> {
        if self.line_ends.start() + idx == 0 {
            self.line_ends.get(idx).copied()
        } else {
            self.line_ends.get(idx).map(|i| i + 1)
        }
    }

    pub fn line_end(&self, idx: usize) -> Option<usize> {
        self.line_ends.get(idx + 1).copied()
    }

    pub fn len(&self) -> usize {
        self.line_ends.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.line_ends.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test building LineIndex with empty string
    #[test]
    fn test_build_empty_string() {
        let index = LineIndex::build("");
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    // Test building LineIndex with single character (no newline)
    #[test]
    fn test_build_single_character_no_newline() {
        let index = LineIndex::build("a");
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with single newline
    #[test]
    fn test_build_single_newline() {
        let index = LineIndex::build("\n");
        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with text ending with newline
    #[test]
    fn test_build_text_ending_with_newline() {
        let index = LineIndex::build("hello\n");
        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with text not ending with newline
    #[test]
    fn test_build_text_not_ending_with_newline() {
        let index = LineIndex::build("hello");
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with multiple lines
    #[test]
    fn test_build_multiple_lines() {
        let index = LineIndex::build("line1\nline2\nline3");
        assert_eq!(index.len(), 3);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with consecutive newlines
    #[test]
    fn test_build_consecutive_newlines() {
        let index = LineIndex::build("line1\n\n\nline2");
        assert_eq!(index.len(), 4);
        assert!(!index.is_empty());
    }

    // Test building LineIndex with only newlines
    #[test]
    fn test_build_only_newlines() {
        let index = LineIndex::build("\n\n\n");
        assert_eq!(index.len(), 4);
        assert!(!index.is_empty());
    }

    // Test slice with full range
    #[test]
    fn test_slice_full_range() {
        let index = LineIndex::build("line1\nline2\nline3");
        let sliced = index.slice(0..index.line_ends.len());
        assert_eq!(sliced.len(), index.len());
    }

    // Test slice with partial range from start
    #[test]
    fn test_slice_partial_range_from_start() {
        let index = LineIndex::build("line1\nline2\nline3");
        let sliced = index.slice(0..2);
        assert_eq!(sliced.len(), 1);
    }

    // Test slice with partial range from middle
    #[test]
    fn test_slice_partial_range_from_middle() {
        let index = LineIndex::build("line1\nline2\nline3");
        let sliced = index.slice(1..3);
        assert_eq!(sliced.len(), 1);
    }

    // Test slice with empty range
    #[test]
    fn test_slice_empty_range() {
        let index = LineIndex::build("line1\nline2\nline3");
        let sliced = index.slice(1..1);
        assert_eq!(sliced.len(), 0);
        assert!(sliced.is_empty());
    }

    // Test line_start for first line (index 0)
    #[test]
    fn test_line_start_first_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_start(0), Some(0));
    }

    // Test line_start for second line
    #[test]
    fn test_line_start_second_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_start(1), Some(6));
    }

    // Test line_start for non-existent line
    #[test]
    fn test_line_start_non_existent_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_start(5), None);
    }

    // Test line_start for empty index
    #[test]
    fn test_line_start_empty_index() {
        let index = LineIndex::build("");
        assert_eq!(index.line_start(0), None);
    }

    // Test line_start with sliced index
    #[test]
    fn test_line_start_with_sliced_index() {
        let index = LineIndex::build("line1\nline2\nline3");
        let sliced = index.slice(1..3);
        assert_eq!(sliced.line_start(0), Some(6));
    }

    // Test line_end for first line
    #[test]
    fn test_line_end_first_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_end(0), Some(5));
    }

    // Test line_end for last line
    #[test]
    fn test_line_end_last_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_end(1), Some(11));
    }

    // Test line_end for non-existent line
    #[test]
    fn test_line_end_non_existent_line() {
        let index = LineIndex::build("hello\nworld");
        assert_eq!(index.line_end(5), None);
    }

    // Test line_end for empty index
    #[test]
    fn test_line_end_empty_index() {
        let index = LineIndex::build("");
        assert_eq!(index.line_end(0), None);
    }

    // Test line_end with text ending in newline
    #[test]
    fn test_line_end_text_ending_with_newline() {
        let index = LineIndex::build("hello\nworld\n");
        assert_eq!(index.line_end(0), Some(5));
        assert_eq!(index.line_end(1), Some(11));
    }

    // Test len with empty string
    #[test]
    fn test_len_empty_string() {
        let index = LineIndex::build("");
        assert_eq!(index.len(), 0);
    }

    // Test len with single line no newline
    #[test]
    fn test_len_single_line_no_newline() {
        let index = LineIndex::build("hello");
        assert_eq!(index.len(), 1);
    }

    // Test len with single line with newline
    #[test]
    fn test_len_single_line_with_newline() {
        let index = LineIndex::build("hello\n");
        assert_eq!(index.len(), 2);
    }

    // Test len with multiple lines
    #[test]
    fn test_len_multiple_lines() {
        let index = LineIndex::build("line1\nline2\nline3");
        assert_eq!(index.len(), 3);
    }

    // Test len with only newlines
    #[test]
    fn test_len_only_newlines() {
        let index = LineIndex::build("\n\n\n");
        assert_eq!(index.len(), 4);
    }

    // Test is_empty with empty string
    #[test]
    fn test_is_empty_with_empty_string() {
        let index = LineIndex::build("");
        assert!(index.is_empty());
    }

    // Test is_empty with non-empty string
    #[test]
    fn test_is_empty_with_non_empty_string() {
        let index = LineIndex::build("hello");
        assert!(!index.is_empty());
    }

    // Test is_empty with single newline
    #[test]
    fn test_is_empty_with_single_newline() {
        let index = LineIndex::build("\n");
        assert!(!index.is_empty());
    }

    // Test is_empty with sliced empty range
    #[test]
    fn test_is_empty_with_sliced_empty_range() {
        let index = LineIndex::build("hello\nworld");
        let sliced = index.slice(1..1);
        assert!(sliced.is_empty());
    }

    // Test line positions with empty lines
    #[test]
    fn test_line_positions_with_empty_lines() {
        let index = LineIndex::build("line1\n\nline3");
        assert_eq!(index.len(), 3);
        assert_eq!(index.line_start(0), Some(0));
        assert_eq!(index.line_end(0), Some(5));
        assert_eq!(index.line_start(1), Some(6));
        assert_eq!(index.line_end(1), Some(6));
        assert_eq!(index.line_start(2), Some(7));
        assert_eq!(index.line_end(2), Some(12));
    }

    // Test with unicode characters
    #[test]
    fn test_with_unicode_characters() {
        let index = LineIndex::build("héllo\nwörld");
        assert_eq!(index.len(), 2);
        assert_eq!(index.line_start(0), Some(0));
        assert_eq!(index.line_start(1), Some(7)); // 'é' is 2 bytes in UTF-8
    }

    // Test clone functionality
    #[test]
    fn test_clone() {
        let index = LineIndex::build("line1\nline2");
        let cloned = index.clone();
        assert_eq!(index.len(), cloned.len());
        assert_eq!(index.line_start(0), cloned.line_start(0));
        assert_eq!(index.line_end(0), cloned.line_end(0));
    }

    // Test with very large input (stress test)
    #[test]
    fn test_with_large_input() {
        let large_text = "line\n".repeat(10000);
        let index = LineIndex::build(&large_text);
        assert_eq!(index.len(), 10001); // 10000 lines + 1 final empty line
        assert_eq!(index.line_start(0), Some(0));
        assert_eq!(index.line_start(1), Some(5));
        assert_eq!(index.line_end(0), Some(4));
    }

    // Test boundary conditions for line_start and line_end
    #[test]
    fn test_boundary_conditions_line_access() {
        let index = LineIndex::build("a\nb\nc");
        // Test accessing exactly at the boundary
        assert_eq!(index.line_start(2), Some(4));
        assert_eq!(index.line_end(2), Some(5));
        // Test accessing beyond boundary
        assert_eq!(index.line_start(3), Some(6));
        assert_eq!(index.line_end(3), None);
    }

    // Test with string containing only whitespace
    #[test]
    fn test_with_whitespace_only() {
        let index = LineIndex::build("   \n\t\n  ");
        assert_eq!(index.len(), 3);
        assert_eq!(index.line_start(0), Some(0));
        assert_eq!(index.line_end(0), Some(3));
        assert_eq!(index.line_start(1), Some(4));
        assert_eq!(index.line_end(1), Some(5));
    }

    // Test slice with out-of-bounds range (should not panic)
    #[test]
    fn test_slice_with_edge_ranges() {
        let index = LineIndex::build("line1\nline2");
        // Test slicing from the last valid index
        let sliced = index.slice(2..3);
        assert_eq!(sliced.len(), 0);
    }

    // Test line_start behavior with sliced index at different positions
    #[test]
    fn test_line_start_sliced_index_edge_cases() {
        let index = LineIndex::build("a\nb\nc\nd");
        let sliced = index.slice(0..2); // Contains indices 0 and 1

        // When slice starts from 0, first line should still start at position from original
        assert_eq!(sliced.line_start(0), Some(0));
        // Second line in slice should give proper offset
        assert_eq!(sliced.line_start(1), Some(2));
    }
}
