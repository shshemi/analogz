#![allow(dead_code)]

use itertools::Itertools;

use crate::containers::{ArcSlice, InvalidIndexError};

type RangeUsize = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct RangeIndex {
    slice: ArcSlice<RangeUsize>,
}

impl RangeIndex {
    pub fn new(ranges: impl IntoIterator<Item = RangeUsize>) -> Self {
        Self {
            slice: ArcSlice::new(ranges.into_iter().collect_vec()),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&RangeUsize> {
        self.slice.get(idx)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slice(&self, rng: RangeUsize) -> Self {
        RangeIndex {
            slice: self.slice.slice(rng),
        }
    }

    pub fn select(
        &self,
        items: impl IntoIterator<Item = usize>,
    ) -> Result<Self, InvalidIndexError> {
        Ok(RangeIndex {
            slice: self.slice.select(items)?,
        })
    }
}

impl FromIterator<RangeUsize> for RangeIndex {
    fn from_iter<T: IntoIterator<Item = RangeUsize>>(iter: T) -> Self {
        Self::new(iter)
    }
}

#[allow(clippy::single_range_in_vec_init)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_empty_ranges() {
        let index = RangeIndex::new(vec![]);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_new_with_single_range() {
        let index = RangeIndex::new([0..5]);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_new_with_multiple_ranges() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_new_with_empty_range() {
        let index = RangeIndex::new(vec![5..5]);
        assert_eq!(index.len(), 1);
        assert_eq!(index.get(0), Some(&(5..5)));
    }

    // Tests for get()
    #[test]
    fn test_get_valid_index() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        assert_eq!(index.get(0), Some(&(0..5)));
        assert_eq!(index.get(1), Some(&(10..15)));
        assert_eq!(index.get(2), Some(&(20..25)));
    }

    #[test]
    fn test_get_invalid_index() {
        let index = RangeIndex::new(vec![0..5, 10..15]);
        assert_eq!(index.get(2), None);
    }

    #[test]
    fn test_get_on_empty_index() {
        let index = RangeIndex::new(vec![]);
        assert_eq!(index.get(0), None);
    }

    #[test]
    fn test_get_boundary_index() {
        let index = RangeIndex::new(vec![0..5, 10..15]);
        assert_eq!(index.get(1), Some(&(10..15)));
        assert_eq!(index.get(2), None);
    }

    // Tests for len()
    #[test]
    fn test_len_empty() {
        let index = RangeIndex::new(vec![]);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_len_single_element() {
        let index = RangeIndex::new(vec![0..5]);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_len_multiple_elements() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35]);
        assert_eq!(index.len(), 4);
    }

    // Tests for is_empty()
    #[test]
    fn test_is_empty_true() {
        let index = RangeIndex::new(vec![]);
        assert!(index.is_empty());
    }

    #[test]
    fn test_is_empty_false() {
        let index = RangeIndex::new(vec![0..5]);
        assert!(!index.is_empty());
    }

    // Tests for slice()
    #[test]
    fn test_slice_full_range() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let sliced = index.slice(0..3);
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.get(0), Some(&(0..5)));
        assert_eq!(sliced.get(2), Some(&(20..25)));
    }

    #[test]
    fn test_slice_partial_range() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35]);
        let sliced = index.slice(1..3);
        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.get(0), Some(&(10..15)));
        assert_eq!(sliced.get(1), Some(&(20..25)));
    }

    #[test]
    fn test_slice_empty_range() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let sliced = index.slice(1..1);
        assert_eq!(sliced.len(), 0);
        assert!(sliced.is_empty());
    }

    #[test]
    fn test_slice_from_start() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let sliced = index.slice(0..2);
        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.get(0), Some(&(0..5)));
        assert_eq!(sliced.get(1), Some(&(10..15)));
    }

    #[test]
    fn test_slice_to_end() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let sliced = index.slice(1..3);
        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.get(0), Some(&(10..15)));
        assert_eq!(sliced.get(1), Some(&(20..25)));
    }

    #[test]
    fn test_slice_single_element() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let sliced = index.slice(1..2);
        assert_eq!(sliced.len(), 1);
        assert_eq!(sliced.get(0), Some(&(10..15)));
    }

    #[test]
    fn test_slice_of_empty_index() {
        let index = RangeIndex::new(vec![]);
        let sliced = index.slice(0..0);
        assert_eq!(sliced.len(), 0);
    }

    // Tests for select()
    #[test]
    fn test_select_valid_indices() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35]);
        let selected = index.select(vec![0, 2]).unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected.get(0), Some(&(0..5)));
        assert_eq!(selected.get(1), Some(&(20..25)));
    }

    #[test]
    fn test_select_single_index() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let selected = index.select(vec![1]).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected.get(0), Some(&(10..15)));
    }

    #[test]
    fn test_select_empty_indices() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let selected = index.select(vec![]).unwrap();
        assert_eq!(selected.len(), 0);
        assert!(selected.is_empty());
    }

    #[test]
    fn test_select_all_indices() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let selected = index.select(vec![0, 1, 2]).unwrap();
        assert_eq!(selected.len(), 3);
        assert_eq!(selected.get(0), Some(&(0..5)));
        assert_eq!(selected.get(1), Some(&(10..15)));
        assert_eq!(selected.get(2), Some(&(20..25)));
    }

    #[test]
    fn test_select_invalid_index() {
        let index = RangeIndex::new(vec![0..5, 10..15]);
        let result = index.select(vec![0, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_out_of_bounds() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let result = index.select(vec![3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_duplicate_indices() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let selected = index.select(vec![1, 1]).unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected.get(0), Some(&(10..15)));
        assert_eq!(selected.get(1), Some(&(10..15)));
    }

    #[test]
    fn test_select_reverse_order() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25]);
        let selected = index.select(vec![2, 1, 0]).unwrap();
        assert_eq!(selected.len(), 3);
        assert_eq!(selected.get(0), Some(&(20..25)));
        assert_eq!(selected.get(1), Some(&(10..15)));
        assert_eq!(selected.get(2), Some(&(0..5)));
    }

    // Tests for FromIterator
    #[test]
    fn test_from_iterator_vec() {
        let index: RangeIndex = vec![0..5, 10..15, 20..25].into_iter().collect();
        assert_eq!(index.len(), 3);
        assert_eq!(index.get(1), Some(&(10..15)));
    }

    #[test]
    fn test_from_iterator_empty() {
        let index: RangeIndex = vec![].into_iter().collect();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_from_iterator_single_element() {
        let index: RangeIndex = vec![5..10].into_iter().collect();
        assert_eq!(index.len(), 1);
        assert_eq!(index.get(0), Some(&(5..10)));
    }

    // Tests for Clone
    #[test]
    fn test_clone_creates_independent_copy() {
        let index = RangeIndex::new(vec![0..5, 10..15]);
        let cloned = index.clone();
        assert_eq!(cloned.len(), index.len());
        assert_eq!(cloned.get(0), index.get(0));
        assert_eq!(cloned.get(1), index.get(1));
    }

    #[test]
    fn test_clone_empty_index() {
        let index = RangeIndex::new(vec![]);
        let cloned = index.clone();
        assert_eq!(cloned.len(), 0);
        assert!(cloned.is_empty());
    }

    // Integration tests
    #[test]
    fn test_slice_then_select() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35, 40..45]);
        let sliced = index.slice(1..4);
        let selected = sliced.select(vec![0, 2]).unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected.get(0), Some(&(10..15)));
        assert_eq!(selected.get(1), Some(&(30..35)));
    }

    #[test]
    fn test_select_then_slice() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35]);
        let selected = index.select(vec![0, 2, 3]).unwrap();
        let sliced = selected.slice(1..3);
        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.get(0), Some(&(20..25)));
        assert_eq!(sliced.get(1), Some(&(30..35)));
    }

    #[test]
    fn test_multiple_slices() {
        let index = RangeIndex::new(vec![0..5, 10..15, 20..25, 30..35, 40..45]);
        let sliced1 = index.slice(1..5);
        let sliced2 = sliced1.slice(1..3);
        assert_eq!(sliced2.len(), 2);
        assert_eq!(sliced2.get(0), Some(&(20..25)));
        assert_eq!(sliced2.get(1), Some(&(30..35)));
    }
}
