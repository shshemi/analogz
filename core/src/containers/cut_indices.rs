use itertools::Itertools;

use std::ops::Range;

use crate::containers::ArcSlice;

#[derive(Debug, Clone)]
pub struct CutIndices {
    indices: ArcSlice<usize>,
}

impl CutIndices {
    pub fn build<T, F>(arr: impl AsRef<[T]>, f: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        let arr = arr.as_ref();
        let indices = std::iter::once(0)
            .chain(
                arr.iter()
                    .enumerate()
                    .filter_map(move |(i, c)| (f(c)).then_some(i)),
            )
            .chain([arr.len()])
            .collect_vec();

        CutIndices {
            indices: indices.into(),
        }
    }

    pub fn build_par<T, F>(arr: impl AsRef<[T]>, f: F) -> Self
    where
        T: Sync,
        F: Fn(&T) -> bool + Send + Clone + 'static,
    {
        let arr = arr.as_ref();
        let indices = std::thread::scope(|scope| {
            let chunk_size = num_cpus::get();
            std::iter::once(0)
                .chain(
                    arr.chunks(chunk_size)
                        .enumerate()
                        .map(move |(idx, slice)| (idx * chunk_size, slice))
                        .map(|(offset, slice)| {
                            let f = f.clone();
                            scope.spawn(move || {
                                slice
                                    .iter()
                                    .enumerate()
                                    .filter_map(move |(i, c)| (f(c)).then_some(offset + i))
                            })
                        })
                        .flat_map(|hndl| hndl.join().unwrap()),
                )
                .chain([arr.len()])
                .collect_vec()
        });

        CutIndices {
            indices: indices.into(),
        }
    }

    pub fn slice(&self, rng: Range<usize>) -> Self {
        CutIndices {
            indices: self.indices.slice(rng.start..rng.end + 1),
        }
    }

    pub fn start(&self, idx: usize) -> Option<usize> {
        if self.indices.start() + idx == 0 {
            self.indices.get(idx).copied()
        } else {
            self.indices.get(idx).map(|i| i + 1)
        }
    }

    pub fn end(&self, idx: usize) -> Option<usize> {
        self.indices.get(idx + 1).copied()
    }

    pub fn len(&self) -> usize {
        self.indices.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test data
    fn test_chars() -> Vec<char> {
        vec!['a', '\n', 'b', 'c', '\n', 'd', '\n']
    }

    fn is_newline(c: &char) -> bool {
        *c == '\n'
    }

    fn is_vowel(c: &char) -> bool {
        matches!(*c, 'a' | 'e' | 'i' | 'o' | 'u')
    }

    fn always_true<T>(_: &T) -> bool {
        true
    }

    fn always_false<T>(_: &T) -> bool {
        false
    }

    // Tests for build()
    #[test]
    fn test_build_empty_array() {
        let empty: Vec<char> = vec![];
        let cut_indices = CutIndices::build(empty, is_newline);

        assert_eq!(cut_indices.len(), 1);
        assert!(!cut_indices.is_empty());
    }

    #[test]
    fn test_build_single_element_matching() {
        let data = vec!['\n'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 2);
        assert!(!cut_indices.is_empty());
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(0));
        assert_eq!(cut_indices.start(1), Some(1));
        assert_eq!(cut_indices.end(1), Some(1));
    }

    #[test]
    fn test_build_single_element_not_matching() {
        let data = vec!['a'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 1);
        assert!(!cut_indices.is_empty());
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(1));
    }

    #[test]
    fn test_build_no_matches() {
        let data = vec!['a', 'b', 'c'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 1);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(3));
    }

    #[test]
    fn test_build_all_matches() {
        let data = vec!['\n', '\n', '\n'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 4); // 0, after each \n, and end
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(0));
        assert_eq!(cut_indices.start(1), Some(1));
        assert_eq!(cut_indices.end(1), Some(1));
        assert_eq!(cut_indices.start(2), Some(2));
        assert_eq!(cut_indices.end(2), Some(2));
        assert_eq!(cut_indices.start(3), Some(3));
        assert_eq!(cut_indices.end(3), Some(3));
    }

    #[test]
    fn test_build_mixed_matches() {
        let data = test_chars(); // ['a', '\n', 'b', 'c', '\n', 'd', '\n']
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 4);
        // Line 0: indices 0-1 (chars 'a')
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(1));
        // Line 1: indices 2-4 (chars 'b', 'c')
        assert_eq!(cut_indices.start(1), Some(2));
        assert_eq!(cut_indices.end(1), Some(4));
        // Line 2: indices 5-5 (chars 'd')
        assert_eq!(cut_indices.start(2), Some(5));
        assert_eq!(cut_indices.end(2), Some(6));
        // Line 3: indices 7-7 (empty line at end)
        assert_eq!(cut_indices.start(3), Some(7));
        assert_eq!(cut_indices.end(3), Some(7));
    }

    #[test]
    fn test_build_first_element_matches() {
        let data = vec!['\n', 'a', 'b'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 2);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(0));
        assert_eq!(cut_indices.start(1), Some(1));
        assert_eq!(cut_indices.end(1), Some(3));
    }

    #[test]
    fn test_build_last_element_matches() {
        let data = vec!['a', 'b', '\n'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 2);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(2));
        assert_eq!(cut_indices.start(1), Some(3));
        assert_eq!(cut_indices.end(1), Some(3));
    }

    // Tests for build_par()
    #[test]
    fn test_build_par_empty_array() {
        let empty: Vec<char> = vec![];
        let cut_indices = CutIndices::build_par(empty, is_newline);

        assert_eq!(cut_indices.len(), 1);
        assert!(!cut_indices.is_empty());
    }

    #[test]
    fn test_build_par_single_element() {
        let data = vec!['\n'];
        let cut_indices = CutIndices::build_par(data, is_newline);

        assert_eq!(cut_indices.len(), 2);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(0));
        assert_eq!(cut_indices.start(1), Some(1));
        assert_eq!(cut_indices.end(1), Some(1));
    }

    #[test]
    fn test_build_par_matches_sequential() {
        let data = test_chars();
        let sequential = CutIndices::build(data.clone(), is_newline);
        let parallel = CutIndices::build_par(data, is_newline);

        assert_eq!(sequential.len(), parallel.len());
        for i in 0..sequential.len() {
            assert_eq!(sequential.start(i), parallel.start(i));
            assert_eq!(sequential.end(i), parallel.end(i));
        }
    }

    #[test]
    fn test_build_par_large_data() {
        let data: Vec<char> = (0..1000)
            .map(|i| if i % 10 == 0 { '\n' } else { 'x' })
            .collect();
        let sequential = CutIndices::build(data.clone(), is_newline);
        let parallel = CutIndices::build_par(data, is_newline);

        assert_eq!(sequential.len(), parallel.len());
        for i in 0..sequential.len() {
            assert_eq!(sequential.start(i), parallel.start(i));
            assert_eq!(sequential.end(i), parallel.end(i));
        }
    }

    // Tests for slice()
    #[test]
    fn test_slice_full_range() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(0..cut_indices.len());

        assert_eq!(sliced.len(), cut_indices.len());
        for i in 0..cut_indices.len() {
            assert_eq!(sliced.start(i), cut_indices.start(i));
            assert_eq!(sliced.end(i), cut_indices.end(i));
        }
    }

    #[test]
    fn test_slice_partial_range() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(1..3);

        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.start(0), cut_indices.start(1));
        assert_eq!(sliced.end(0), cut_indices.end(1));
        assert_eq!(sliced.start(1), cut_indices.start(2));
        assert_eq!(sliced.end(1), cut_indices.end(2));
    }

    #[test]
    fn test_slice_empty_range() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(2..2);

        assert_eq!(sliced.len(), 0);
        assert!(sliced.is_empty());
    }

    #[test]
    fn test_slice_single_element() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(1..2);

        assert_eq!(sliced.len(), 1);
        assert_eq!(sliced.start(0), cut_indices.start(1));
        assert_eq!(sliced.end(0), cut_indices.end(1));
    }

    // Tests for start()
    #[test]
    fn test_start_valid_indices() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.start(1), Some(2));
        assert_eq!(cut_indices.start(2), Some(5));
        assert_eq!(cut_indices.start(3), Some(7));
    }

    #[test]
    fn test_start_out_of_bounds() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.start(100), None);
    }

    #[test]
    fn test_start_empty_cut_indices() {
        let empty: Vec<char> = vec![];
        let cut_indices = CutIndices::build(empty, is_newline);

        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(0));
    }

    #[test]
    fn test_start_first_index_special_case() {
        // Testing the special case where start() + idx == 0
        let data = vec!['a', '\n', 'b'];
        let cut_indices = CutIndices::build(data, is_newline);

        // For index 0, it should return the value directly (not +1)
        assert_eq!(cut_indices.start(0), Some(0));
        // For other indices, it should return value + 1
        assert_eq!(cut_indices.start(1), Some(2));
    }

    // Tests for end()
    #[test]
    fn test_end_valid_indices() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.end(0), Some(1));
        assert_eq!(cut_indices.end(1), Some(4));
        assert_eq!(cut_indices.end(2), Some(6));
        assert_eq!(cut_indices.end(3), Some(7));
    }

    #[test]
    fn test_end_out_of_bounds() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.end(100), None);
    }

    #[test]
    fn test_end_empty_cut_indices() {
        let empty: Vec<char> = vec![];
        let cut_indices = CutIndices::build(empty, is_newline);

        assert_eq!(cut_indices.end(0), Some(0));
    }

    #[test]
    fn test_end_last_valid_index() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let last_idx = cut_indices.len() - 1;

        assert_eq!(cut_indices.end(last_idx), Some(7));
        assert_eq!(cut_indices.end(last_idx + 1), None);
    }

    // Tests for len()
    #[test]
    fn test_len_empty() {
        let empty: Vec<char> = vec![];
        let cut_indices = CutIndices::build(empty, is_newline);

        assert_eq!(cut_indices.len(), 1);
    }

    #[test]
    fn test_len_single_element() {
        let data = vec!['a'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 1);
    }

    #[test]
    fn test_len_multiple_elements() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 4);
    }

    #[test]
    fn test_len_after_slice() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(1..3);

        assert_eq!(sliced.len(), 2);
    }

    // Tests for is_empty()

    #[test]
    fn test_is_empty_false() {
        let data = vec!['a'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert!(!cut_indices.is_empty());
    }

    #[test]
    fn test_is_empty_after_empty_slice() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(2..2);

        assert!(sliced.is_empty());
    }

    // Integration tests combining multiple operations
    #[test]
    fn test_slice_then_query() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let sliced = cut_indices.slice(1..3);

        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.start(0), Some(2));
        assert_eq!(sliced.end(0), Some(4));
        assert_eq!(sliced.start(1), Some(5));
        assert_eq!(sliced.end(1), Some(6));
        assert_eq!(sliced.start(2), Some(7));
        assert_eq!(sliced.end(2), None);
    }

    #[test]
    fn test_different_predicate_functions() {
        let data = vec!['a', 'e', 'b', 'i', 'c'];
        let vowel_cuts = CutIndices::build(data.clone(), is_vowel);

        assert_eq!(vowel_cuts.len(), 4);
        // Should find vowels at positions 0, 1, 3
        assert_eq!(vowel_cuts.start(0), Some(0));
        assert_eq!(vowel_cuts.end(0), Some(0));
        assert_eq!(vowel_cuts.start(1), Some(1));
        assert_eq!(vowel_cuts.end(1), Some(1));
        assert_eq!(vowel_cuts.start(2), Some(2));
        assert_eq!(vowel_cuts.end(2), Some(3));
        assert_eq!(vowel_cuts.start(3), Some(4));
        assert_eq!(vowel_cuts.end(3), Some(5));
    }

    #[test]
    fn test_always_true_predicate() {
        let data = vec!['a', 'b', 'c'];
        let cut_indices = CutIndices::build(data, always_true);

        // Every element matches, so we get n+1 segments (each element is a separator)
        assert_eq!(cut_indices.len(), 4);
    }

    #[test]
    fn test_always_false_predicate() {
        let data = vec!['a', 'b', 'c'];
        let cut_indices = CutIndices::build(data, always_false);

        // No matches, so we get 1 segment covering the whole array
        assert_eq!(cut_indices.len(), 1);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(3));
    }

    // Edge case: consecutive matches
    #[test]
    fn test_consecutive_matches() {
        let data = vec!['a', '\n', '\n', 'b'];
        let cut_indices = CutIndices::build(data, is_newline);

        assert_eq!(cut_indices.len(), 3);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(1));
        assert_eq!(cut_indices.start(1), Some(2));
        assert_eq!(cut_indices.end(1), Some(2));
        assert_eq!(cut_indices.start(2), Some(3));
        assert_eq!(cut_indices.end(2), Some(4));
    }

    // Test with different data types
    #[test]
    fn test_with_integers() {
        let data = vec![1, 5, 3, 5, 2];
        let cut_indices = CutIndices::build(data, |&x| x == 5);

        assert_eq!(cut_indices.len(), 3);
        assert_eq!(cut_indices.start(0), Some(0));
        assert_eq!(cut_indices.end(0), Some(1));
        assert_eq!(cut_indices.start(1), Some(2));
        assert_eq!(cut_indices.end(1), Some(3));
        assert_eq!(cut_indices.start(2), Some(4));
        assert_eq!(cut_indices.end(2), Some(5));
    }

    // Test clone functionality
    #[test]
    fn test_clone() {
        let data = test_chars();
        let cut_indices = CutIndices::build(data, is_newline);
        let cloned = cut_indices.clone();

        assert_eq!(cut_indices.len(), cloned.len());
        for i in 0..cut_indices.len() {
            assert_eq!(cut_indices.start(i), cloned.start(i));
            assert_eq!(cut_indices.end(i), cloned.end(i));
        }
    }
}
