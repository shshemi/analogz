use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::containers::ArcSlice;

pub trait FindAll<T> {
    fn find_all(&self, predicate: impl Fn(&T) -> bool) -> ArcSlice<usize>;
}

pub trait ParFindAll<T> {
    fn par_find_all(
        &self,
        predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
    ) -> ArcSlice<usize>
    where
        T: Sync;
}

impl<C, T> FindAll<T> for C
where
    C: AsRef<[T]>,
{
    fn find_all(&self, predicate: impl Fn(&T) -> bool) -> ArcSlice<usize> {
        ArcSlice::new(
            self.as_ref()
                .iter()
                .enumerate()
                .filter_map(|(i, v)| predicate(v).then_some(i))
                .collect::<Vec<_>>(),
        )
    }
}

impl<C, T> ParFindAll<T> for C
where
    C: AsRef<[T]>,
{
    fn par_find_all(
        &self,
        predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
    ) -> ArcSlice<usize>
    where
        T: Sync,
    {
        ArcSlice::new(
            self.as_ref()
                .par_iter()
                .enumerate()
                .filter_map(|(i, v)| predicate(v).then_some(i))
                .collect::<Vec<_>>(),
        )
    }
}
