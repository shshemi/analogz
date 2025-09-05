use std::collections::VecDeque;

#[derive(Debug)]
pub struct RoundRobin<T> {
    queue: VecDeque<T>,
}

impl<I, T> Iterator for RoundRobin<T>
where
    T: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut iter) = self.queue.pop_front() {
            if let Some(item) = iter.next() {
                self.queue.push_back(iter);
                return Some(item);
            }
        }
        None
    }
}

pub trait IntoRoundRobin {
    type Item;

    fn round_robin(self) -> RoundRobin<Self::Item>;
}

impl<I, T> IntoRoundRobin for T
where
    T: IntoIterator<Item = I>,
{
    type Item = I;

    fn round_robin(self) -> RoundRobin<Self::Item> {
        RoundRobin {
            queue: self.into_iter().collect(),
        }
    }
}
