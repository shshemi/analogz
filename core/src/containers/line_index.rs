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
                std::iter::once(0)
                    .chain(
                        chunk_str(corpus, num_cpus::get())
                            .map(|(offset, slice)| scope.spawn(move || new_lines(slice, offset)))
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

fn chunk_str(slice: &str, count: usize) -> impl Iterator<Item = (usize, &[u8])> {
    let slice_len = (slice.len() / count).max(1);
    slice
        .as_bytes()
        .chunks(slice_len)
        .enumerate()
        .map(move |(idx, slice)| (idx * slice_len, slice))
}

fn new_lines<'a>(slice: &'a [u8], offset: usize) -> impl Iterator<Item = usize> + 'a {
    slice
        .iter()
        .enumerate()
        .filter_map(move |(i, c)| (*c == b'\n').then_some(offset + i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lines_function() {
        let data = "a\nb\nc".as_bytes();
        let newlines: Vec<_> = new_lines(data, 10).collect();
        assert_eq!(newlines, vec![11, 13]); // Position 10+1 and 10+3
    }

    #[test]
    fn test_chunk_function() {
        let text = "abcdefghijklmnopqr";
        let chunks: Vec<_> = chunk_str(text, 3).collect();

        // Test the actual slices
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].0, 0); // First chunk starts at offset 0
        assert_eq!(std::str::from_utf8(chunks[0].1).unwrap(), "abcdef");

        assert_eq!(chunks[1].0, 6); // Second chunk starts at offset 6
        assert_eq!(std::str::from_utf8(chunks[1].1).unwrap(), "ghijkl");

        assert_eq!(chunks[2].0, 12); // Third chunk starts at offset 12
        assert_eq!(std::str::from_utf8(chunks[2].1).unwrap(), "mnopqr");

        // Check that the chunks cover the entire string
        let mut reconstructed = String::new();
        for (_, chunk) in chunks {
            reconstructed.push_str(std::str::from_utf8(chunk).unwrap());
        }
        assert_eq!(reconstructed, text);
    }
}
