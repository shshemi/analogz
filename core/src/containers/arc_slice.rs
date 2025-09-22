use std::{
    ops::{Deref, RangeBounds},
    sync::Arc,
};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct ArcSlice<T> {
    slice: Arc<[T]>,
    start: usize,
    end: usize,
}

impl<T> ArcSlice<T> {
    pub fn new(slice: impl Into<Arc<[T]>>) -> Self {
        let aslice = slice.into();
        Self {
            start: 0,
            end: aslice.len(),
            slice: aslice,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        let idx = self.start + idx;
        if idx < self.end {
            self.slice.get(idx)
        } else {
            None
        }
    }

    pub fn slice(&self, rng: impl RangeBounds<usize>) -> Self {
        let start = match rng.start_bound() {
            std::ops::Bound::Included(i) => self.start + i,
            std::ops::Bound::Excluded(i) => self.start + i + 1,
            std::ops::Bound::Unbounded => self.start,
        }
        .clamp(self.start, self.end);
        let end = match rng.end_bound() {
            std::ops::Bound::Included(i) => self.start + i + 1,
            std::ops::Bound::Excluded(i) => self.start + i,
            std::ops::Bound::Unbounded => self.end,
        }
        .clamp(self.start, self.end);
        // let start = (self.start + rng.start).min(self.end);
        // let end = (self.start + rng.end).min(self.end);

        Self {
            slice: self.slice.clone(),
            start,
            end,
        }
    }

    pub fn select(&self, items: impl IntoIterator<Item = usize>) -> Self
    where
        T: Clone,
    {
        ArcSlice::new(
            items
                .into_iter()
                .filter_map(|idx| self.get(idx).cloned())
                .collect_vec(),
        )
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    pub fn as_slice(&self) -> &[T] {
        &self.slice[self.start..self.end]
    }
}

impl<T, C> From<C> for ArcSlice<T>
where
    C: Into<Arc<[T]>>,
{
    fn from(value: C) -> Self {
        Self::new(value)
    }
}

impl<T> FromIterator<T> for ArcSlice<T> {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        ArcSlice::new(iter.into_iter().collect_vec())
    }
}

impl<T> AsRef<[T]> for ArcSlice<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> Deref for ArcSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(slice.start(), 0);
        assert_eq!(slice.end(), 5);
        assert_eq!(slice.len(), 5);
    }

    #[test]
    fn test_from() {
        let data = vec![1, 2, 3, 4, 5];
        let slice: ArcSlice<_> = data.into();
        assert_eq!(slice.start(), 0);
        assert_eq!(slice.end(), 5);
        assert_eq!(slice.len(), 5);
    }

    #[test]
    fn test_get_valid_index() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(slice.get(0), Some(&1));
        assert_eq!(slice.get(4), Some(&5));
    }

    #[test]
    fn test_get_invalid_index() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(slice.get(5), None);
    }

    #[test]
    fn test_slice_within_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        let subslice = slice.slice(1..4);
        assert_eq!(subslice.start(), 1);
        assert_eq!(subslice.end(), 4);
        assert_eq!(subslice.len(), 3);
        assert_eq!(subslice.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_slice_exceeding_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        let subslice = slice.slice(3..10);
        assert_eq!(subslice.start(), 3);
        assert_eq!(subslice.end(), 5);
        assert_eq!(subslice.len(), 2);
        assert_eq!(subslice.as_slice(), &[4, 5]);
    }

    #[test]
    fn test_slice_of_slice() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        let subslice1 = slice.slice(1..4);
        let subslice2 = subslice1.slice(1..2);
        assert_eq!(subslice2.start(), 2);
        assert_eq!(subslice2.end(), 3);
        assert_eq!(subslice2.as_slice(), &[3]);
    }

    #[test]
    fn test_len() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(slice.len(), 5);

        let empty_data: Vec<i32> = vec![];
        let empty_slice = ArcSlice::new(empty_data);
        assert_eq!(empty_slice.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert!(!slice.is_empty());

        let empty_data: Vec<i32> = vec![];
        let empty_slice = ArcSlice::new(empty_data);
        assert!(empty_slice.is_empty());
    }

    #[test]
    fn test_as_slice() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(slice.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_as_ref() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        let slice_ref: &[i32] = slice.as_ref();
        assert_eq!(slice_ref, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deref() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        assert_eq!(&slice[0..3], &[1, 2, 3]);
    }

    #[test]
    fn test_empty_slice() {
        let data = vec![1, 2, 3];
        let slice = ArcSlice::new(data);
        let empty_slice = slice.slice(1..1);
        assert_eq!(empty_slice.start(), 1);
        assert_eq!(empty_slice.end(), 1);
        assert_eq!(empty_slice.len(), 0);
        assert!(empty_slice.is_empty());
    }

    #[test]
    fn test_arc_sharing() {
        let data = vec![1, 2, 3, 4, 5];
        let slice1 = ArcSlice::new(data);
        let slice2 = slice1.slice(1..3);
        let slice3 = slice1.slice(3..5);

        assert_eq!(slice1.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(slice2.as_slice(), &[2, 3]);
        assert_eq!(slice3.as_slice(), &[4, 5]);

        // All slices should share the same Arc
        assert!(Arc::ptr_eq(&slice1.slice, &slice2.slice));
        assert!(Arc::ptr_eq(&slice1.slice, &slice3.slice));
    }

    #[test]
    fn test_clone() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = ArcSlice::new(data);
        let clone = slice.clone();

        assert_eq!(slice.as_slice(), clone.as_slice());
        assert_eq!(slice.start(), clone.start());
        assert_eq!(slice.end(), clone.end());

        // Both should share the same Arc
        assert!(Arc::ptr_eq(&slice.slice, &clone.slice));
    }
}
