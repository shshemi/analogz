use crate::containers::traits::Find;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, RangeBounds},
    sync::Arc,
};

#[derive(Clone, Hash)]
pub struct ArcStr {
    astr: Arc<str>,
    start: usize,
    end: usize,
}

impl Debug for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArcStr({:?})", self.as_str())
    }
}

impl Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl<T> PartialEq<T> for ArcStr
where
    T: Deref<Target = str>,
{
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other.deref())
    }
}

impl Eq for ArcStr {}

impl<T> PartialOrd<T> for ArcStr
where
    T: Deref<Target = str>,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl Ord for ArcStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other)
    }
}

impl ArcStr {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        let astr = value.into();
        let end = astr.len();

        Self {
            astr,
            start: 0,
            end,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
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
        Self {
            astr: Arc::clone(&self.astr),
            start,
            end,
        }
    }

    pub fn split_at(&self, idx: usize) -> (Self, Self) {
        (self.slice(..idx), self.slice(idx..))
    }

    pub fn split_at_two(&self, idx1: usize, idx2: usize) -> (Self, Self, Self) {
        (
            self.slice(..idx1),
            self.slice(idx1..idx2),
            self.slice(idx2..),
        )
    }

    pub fn find<F: Find>(&self, f: F) -> Option<Self> {
        f.find(self)
    }

    pub fn as_str(&self) -> &str {
        &self.astr[self.start..self.end]
    }

    pub fn contains(&self, other: ArcStr) -> bool {
        (Arc::ptr_eq(&self.astr, &other.astr) && self.start <= other.start && other.end <= self.end)
            || self.as_str().contains(other.as_str())
    }

    /// Returns the relative position (as an `isize`) of another `ArcStr`'s start
    /// index with respect to this `ArcStr`'s start index, if both slices refer to
    /// the same underlying `Arc<str>`. If they do not, returns `None`.
    ///
    /// The result is positive if `other` starts after `self`, negative if it starts
    /// before, and zero if they start at the same position.
    ///
    /// # Example
    /// ```
    /// use analogz::containers::ArcStr;
    /// let base = ArcStr::new("hello world");
    /// let left = base.slice(0..5); // "hello"
    /// let right = base.slice(6..); // "world"
    /// assert_eq!(left.relative_position(&right), Some(6));
    /// assert_eq!(right.relative_position(&left), Some(-6));
    /// assert_eq!(left.relative_position(&left), Some(0));
    ///
    /// let unrelated = ArcStr::new("other");
    /// assert_eq!(left.relative_position(&unrelated), None);
    /// ```
    pub fn relative_position(&self, other: &ArcStr) -> Option<isize> {
        Arc::ptr_eq(&self.astr, &other.astr).then_some(other.start as isize - self.start as isize)
    }
}

impl<C> From<C> for ArcStr
where
    C: Into<Arc<str>>,
{
    fn from(value: C) -> Self {
        ArcStr::new(value)
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_from_string() {
        let s = "hello world";
        let arc_str = ArcStr::new(s);
        assert_eq!(arc_str.as_str(), s);
        assert_eq!(arc_str.start(), 0);
        assert_eq!(arc_str.end(), s.len());
    }

    #[test]
    fn test_slice_with_included_range() {
        let arc_str = ArcStr::new("hello world");
        let sliced = arc_str.slice(1..5);
        assert_eq!(sliced.as_str(), "ello");
        assert_eq!(sliced.start(), 1);
        assert_eq!(sliced.end(), 5);
    }

    #[test]
    fn test_slice_with_inclusive_range() {
        let arc_str = ArcStr::new("hello world");
        let sliced = arc_str.slice(1..=4);
        assert_eq!(sliced.as_str(), "ello");
    }

    #[test]
    fn test_slice_with_start_bound_only() {
        let arc_str = ArcStr::new("hello world");
        let sliced = arc_str.slice(6..);
        assert_eq!(sliced.as_str(), "world");
    }

    #[test]
    fn test_slice_with_end_bound_only() {
        let arc_str = ArcStr::new("hello world");
        let sliced = arc_str.slice(..5);
        assert_eq!(sliced.as_str(), "hello");
    }

    #[test]
    fn test_slice_with_unbounded_range() {
        let arc_str = ArcStr::new("hello world");
        let sliced = arc_str.slice(..);
        assert_eq!(sliced.as_str(), "hello world");
    }

    #[test]
    fn test_slice_with_range_exceeding_bounds() {
        let arc_str = ArcStr::new("hello");
        let sliced = arc_str.slice(2..10);
        assert_eq!(sliced.as_str(), "llo");
    }

    #[test]
    fn test_slice_of_slice() {
        let arc_str = ArcStr::new("hello world");
        let sliced1 = arc_str.slice(6..);
        let sliced2 = sliced1.slice(1..4);
        assert_eq!(sliced2.as_str(), "orl");
    }

    #[test]
    fn test_split_at() {
        let arc_str = ArcStr::new("hello world");
        let (left, right) = arc_str.split_at(5);
        assert_eq!(left.as_str(), "hello");
        assert_eq!(right.as_str(), " world");
    }

    #[test]
    fn test_split_at_beginning() {
        let arc_str = ArcStr::new("hello");
        let (left, right) = arc_str.split_at(0);
        assert_eq!(left.as_str(), "");
        assert_eq!(right.as_str(), "hello");
    }

    #[test]
    fn test_split_at_end() {
        let arc_str = ArcStr::new("hello");
        let (left, right) = arc_str.split_at(5);
        assert_eq!(left.as_str(), "hello");
        assert_eq!(right.as_str(), "");
    }

    #[test]
    fn test_as_str() {
        let s = "hello world";
        let arc_str = ArcStr::new(s);
        assert_eq!(arc_str.as_str(), s);
    }

    #[test]
    fn test_as_ref() {
        let s = "hello world";
        let arc_str = ArcStr::new(s);
        let s_ref: &str = arc_str.as_ref();
        assert_eq!(s_ref, s);
    }

    #[test]
    fn test_deref() {
        let arc_str = ArcStr::new("hello");
        assert_eq!(arc_str.len(), 5);
        assert_eq!(
            arc_str.chars().collect::<Vec<_>>(),
            vec!['h', 'e', 'l', 'l', 'o']
        );
    }

    #[test]
    fn test_from_string() {
        let s = "hello".to_string();
        let arc_str = ArcStr::from(s);
        assert_eq!(arc_str.as_str(), "hello");
    }

    #[test]
    fn test_from_str() {
        let arc_str = ArcStr::from("hello");
        assert_eq!(arc_str.as_str(), "hello");
    }

    #[test]
    fn test_from_arc_str() {
        let s = Arc::from("hello world");
        let arc_str = ArcStr::from(s);
        assert_eq!(arc_str.as_str(), "hello world");
    }

    #[test]
    fn test_clone() {
        let arc_str = ArcStr::new("hello");
        let cloned = arc_str.clone();
        assert_eq!(arc_str, cloned);
    }

    #[test]
    fn test_partial_eq_with_str() {
        let arc_str = ArcStr::new("hello");
        assert_eq!(arc_str.as_str(), "hello");
        assert_ne!(arc_str.as_str(), "world");
    }

    #[test]
    fn test_eq_between_arc_strs() {
        let arc_str1 = ArcStr::new("hello");
        let arc_str2 = ArcStr::new("hello");
        let arc_str3 = ArcStr::new("world");
        assert_eq!(arc_str1, arc_str2);
        assert_ne!(arc_str1, arc_str3);
    }

    #[test]
    fn test_ord_between_arc_strs() {
        let arc_str1 = ArcStr::new("apple");
        let arc_str2 = ArcStr::new("banana");
        assert!(arc_str1 < arc_str2);
    }

    #[test]
    fn test_debug_formatting() {
        let arc_str = ArcStr::new("hello");
        let debug_str = format!("{arc_str:?}");
        assert!(debug_str.contains("hello"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let arc_str = ArcStr::new("hello");
        map.insert(arc_str.clone(), 42);
        assert_eq!(map.get(&arc_str), Some(&42));
    }
}
