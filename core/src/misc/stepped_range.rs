pub struct SteppedRange {
    next: usize,
    end: usize,
    step: usize,
}

impl SteppedRange {
    pub fn new(start: usize, end: usize, step: usize) -> Self {
        if start >= end {
            panic!("Invalid range {start}..{end}")
        }
        if step == 0 {
            panic!("Invalid step {step}")
        }
        Self {
            next: start,
            end,
            step,
        }
    }
}

impl Iterator for SteppedRange {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.end {
            let next = self.next;
            self.next += self.step;
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_range() {
        let sr = SteppedRange::new(0, 5, 1);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_step_greater_than_one() {
        let sr = SteppedRange::new(0, 10, 2);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    #[should_panic]
    fn test_start_equals_end_panics() {
        SteppedRange::new(5, 5, 1);
    }

    #[test]
    #[should_panic]
    fn test_start_greater_than_end_panics() {
        SteppedRange::new(10, 5, 1);
    }

    #[test]
    #[should_panic]
    fn test_step_zero_panics() {
        let sr = SteppedRange::new(0, 3, 0);
    }

    #[test]
    fn test_single_element_range() {
        let sr = SteppedRange::new(2, 3, 1);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_next_returns_none_when_exhausted() {
        let mut sr = SteppedRange::new(0, 2, 1);
        assert_eq!(sr.next(), Some(0));
        assert_eq!(sr.next(), Some(1));
        assert_eq!(sr.next(), None);
        assert_eq!(sr.next(), None);
    }

    #[test]
    fn test_large_step_skips_over_end() {
        let sr = SteppedRange::new(0, 5, 10);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_non_divisible_step() {
        let sr = SteppedRange::new(0, 10, 3);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0, 3, 6, 9]);
    }

    #[test]
    fn test_range_with_step_equal_to_range_length() {
        let sr = SteppedRange::new(0, 5, 5);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_range_with_step_one_less_than_range_length() {
        let sr = SteppedRange::new(0, 5, 4);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![0, 4]);
    }

    #[test]
    fn test_range_with_start_nonzero() {
        let sr = SteppedRange::new(3, 8, 2);
        let result: Vec<_> = sr.collect();
        assert_eq!(result, vec![3, 5, 7]);
    }

    #[test]
    fn test_empty_range() {
        let result = std::panic::catch_unwind(|| {
            SteppedRange::new(10, 10, 1);
        });
        assert!(result.is_err());
    }
}
