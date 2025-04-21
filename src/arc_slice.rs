use std::{
    ops::{Deref, Range},
    sync::Arc,
};

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

    pub fn slice(&self, rng: Range<usize>) -> Self {
        let start = (self.start + rng.start).min(self.end);
        let end = (self.start + rng.end).min(self.end);

        Self {
            slice: self.slice.clone(),
            start,
            end,
        }
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
