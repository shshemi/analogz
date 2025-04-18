use std::{ops::Range, sync::Arc};

use itertools::Itertools;
use rayon::prelude::*;

/// A cheap-to-clone container for storage and retrieval of log lines.
///
/// `LogBuf` stores the full log content as a single string, and maintains
/// an index of line endings for quick access to individual lines. The line
/// indices are created in parallel when handling large strings.
///
/// # Examples
///
/// ```
/// use analogz::container::LogBuf;
///
/// let logs = LogBuf::new("line 1\nline 2\nline 3".to_string());
/// assert_eq!(logs.len(), 3);
///
/// // Iterate through all lines
/// for line in logs.iter() {
///     println!("{}", line.as_str());
/// }
///
/// // Access a specific line
/// if let Some(line) = logs.get(1) {
///     assert_eq!(line.as_str(), "line 2");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct LogBuf {
    buffer: Arc<str>,
    lines: Arc<[usize]>,
    start: usize,
    end: usize,
}

impl LogBuf {
    /// Creates a new `LogBuf` from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The string content to be stored in the buffer
    ///
    /// # Returns
    ///
    /// A new `LogBuf` instance containing the provided content
    ///
    pub fn new(content: String) -> LogBuf {
        let lines: Arc<[usize]> = if content.is_empty() {
            Default::default()
        } else {
            chunk_str(&content, num_cpus::get())
                .par_bridge()
                .flat_map(|(offset, slice)| new_lines(slice, offset).par_bridge())
                .collect::<Vec<_>>()
                .into_iter()
                .sorted()
                .chain(std::iter::once(content.len()))
                .collect()
        };

        LogBuf {
            buffer: Arc::from(content),
            end: lines.len(),
            lines,
            start: 0,
        }
    }

    /// Returns the underlying string content as `&str`.
    ///
    /// # Returns
    ///
    /// A `&str` containing the entire log content
    pub fn as_str(&self) -> &str {
        let start = if self.start == 0 {
            0
        } else if let Some(start) = self.lines.get(self.start - 1) {
            start + 1
        } else {
            0
        };
        let end = if self.end == 0 {
            0
        } else {
            self.lines.get(self.end - 1).copied().unwrap_or(0)
        };
        &self.buffer[start..end]
    }

    /// Returns the number of lines in the log buffer.
    ///
    /// # Returns
    ///
    /// The total number of lines in the log buffer
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Checks if the log buffer is empty (contains no lines).
    ///
    /// # Returns
    ///
    /// `true` if the log buffer contains no lines, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the line at the given index.
    ///
    /// # Arguments
    ///
    /// * `idx` - The line index
    ///
    /// # Returns
    ///
    /// * `Some(Line)` for valid indices
    /// * `None` for invalid indices
    pub fn get(&self, idx: usize) -> Option<Line> {
        let idx = self.start + idx;
        if idx < self.end {
            let (start, end) = if idx == 0 {
                (0, *self.lines.first()?)
            } else {
                (self.lines.get(idx - 1)? + 1, *self.lines.get(idx)?)
            };
            Some(Line {
                slice: &self.buffer[start..end],
                start,
                end,
            })
        } else {
            None
        }
    }

    /// Returns a slice of the log buffer for the given range of lines.
    ///
    /// # Arguments
    ///
    /// * `rng` - A range of line indices to include in the slice
    ///
    /// # Returns
    ///
    /// A new `LogBuf` containing only the lines in the specified range
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::container::LogBuf;
    ///
    /// let logs = LogBuf::new("line 1\nline 2\nline 3\nline 4".to_string());
    /// let middle_lines = logs.slice(1..3);
    /// assert_eq!(middle_lines.len(), 2);
    /// assert_eq!(middle_lines.get(0).unwrap().as_str(), "line 2");
    /// assert_eq!(middle_lines.get(1).unwrap().as_str(), "line 3");
    /// ```
    pub fn slice(&self, rng: Range<usize>) -> LogBuf {
        Self {
            buffer: self.buffer.clone(),
            lines: self.lines.clone(),
            start: (self.start + rng.start).min(self.end),
            end: (self.start + rng.end).min(self.end),
        }
    }

    /// Returns an iterator over all lines in the log buffer.
    ///
    /// # Returns
    ///
    /// An iterator that yields each line in the buffer as a `Line` struct
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::container::LogBuf;
    ///
    /// let logs = LogBuf::new("line 1\nline 2\nline 3".to_string());
    ///
    /// for line in logs.iter() {
    ///     println!("{}", line.as_str());
    /// }
    /// ```
    pub fn iter(&self) -> LineIter {
        LineIter {
            buffer: self,
            idx: 0,
        }
    }
}

/// Iterator over the lines in a `LogBuf`.
///
/// Created by the `LogBuf::iter()` or `LogBuf::iter_from()` methods.
#[derive(Debug)]
pub struct LineIter<'a> {
    buffer: &'a LogBuf,
    idx: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.get(self.idx).inspect(|_| {
            self.idx += 1;
        })
    }
}

/// A cheap-to-clone structure to epresents a log buffer line.
///
/// Each `Line` contains a reference to the original string slice,
/// as well as the start and end positions within the original buffer.
#[derive(Debug, Clone)]
pub struct Line<'a> {
    slice: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Line<'a> {
    /// Returns the slice containing the line.
    pub fn as_str(&self) -> &'a str {
        self.slice
    }

    /// Returns the start position.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position.
    pub fn end(&self) -> usize {
        self.end
    }
}

impl<'a> AsRef<str> for Line<'a> {
    fn as_ref(&self) -> &'a str {
        self.slice
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

fn new_lines(slice: &[u8], offset: usize) -> impl Iterator<Item = usize> {
    slice
        .iter()
        .enumerate()
        .filter_map(move |(i, c)| (*c == b'\n').then_some(offset + i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buffer = LogBuf::new(String::new());
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.as_str(), "");
        assert!(buffer.get(0).is_none());
        assert_eq!(buffer.iter().count(), 0);
    }

    #[test]
    fn test_single_line() {
        let content = "single line".to_string();
        let buffer = LogBuf::new(content.clone());

        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.as_str(), content);

        let line = buffer.get(0).unwrap();
        assert_eq!(line.as_str(), "single line");
        assert_eq!(line.start(), 0);
        assert_eq!(line.end(), 11);

        assert_eq!(buffer.iter().count(), 1);
        assert_eq!(buffer.iter().next().unwrap().as_str(), "single line");
    }

    #[test]
    fn test_multiple_lines() {
        let content = "line 1\nline 2\nline 3".to_string();
        let buffer = LogBuf::new(content.clone());

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.as_str(), content);

        let line1 = buffer.get(0).unwrap();
        assert_eq!(line1.as_str(), "line 1");
        assert_eq!(line1.start(), 0);
        assert_eq!(line1.end(), 6);

        let line2 = buffer.get(1).unwrap();
        assert_eq!(line2.as_str(), "line 2");
        assert_eq!(line2.start(), 7);
        assert_eq!(line2.end(), 13);

        let line3 = buffer.get(2).unwrap();
        assert_eq!(line3.as_str(), "line 3");
        assert_eq!(line3.start(), 14);
        assert_eq!(line3.end(), 20);

        let lines: Vec<_> = buffer.iter().map(|l| l.as_str().to_string()).collect();
        assert_eq!(lines, vec!["line 1", "line 2", "line 3"]);
    }

    #[test]
    fn test_line_as_ref() {
        let content = "test line".to_string();
        let buffer = LogBuf::new(content);
        let line = buffer.get(0).unwrap();

        // Test AsRef<str> implementation
        let str_ref: &str = line.as_ref();
        assert_eq!(str_ref, "test line");
    }

    #[test]
    fn test_trailing_newline() {
        let content = "line 1\nline 2\n".to_string();
        let buffer = LogBuf::new(content);

        assert_eq!(buffer.len(), 3); // Two explicit lines plus empty line at end

        let lines: Vec<_> = buffer.iter().map(|l| l.as_str().to_string()).collect();
        assert_eq!(lines, vec!["line 1", "line 2", ""]);
    }

    #[test]
    fn test_consecutive_newlines() {
        let content = "line 1\n\nline 3".to_string();
        let buffer = LogBuf::new(content);

        assert_eq!(buffer.len(), 3);

        let lines: Vec<_> = buffer.iter().map(|l| l.as_str().to_string()).collect();
        assert_eq!(lines, vec!["line 1", "", "line 3"]);
    }

    #[test]
    fn test_large_content() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("Line number {}\n", i));
        }

        let buffer = LogBuf::new(content.clone());
        assert_eq!(buffer.len(), 1001); // 1000 lines + empty line at end

        // Check random lines
        let line42 = buffer.get(42).unwrap();
        assert_eq!(line42.as_str(), "Line number 42");

        let line999 = buffer.get(999).unwrap();
        assert_eq!(line999.as_str(), "Line number 999");
    }

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

    #[test]
    fn test_slice() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = LogBuf::new(content);

        // Test full range slice
        let full_slice = buffer.slice(0..5);
        assert_eq!(full_slice.len(), 5);
        assert_eq!(full_slice.get(0).unwrap().as_str(), "line 1");
        assert_eq!(full_slice.get(4).unwrap().as_str(), "line 5");

        // Test slice
        let partial_slice = buffer.slice(1..4);
        assert_eq!(partial_slice.len(), 3);
        // Test that as_str returns the expected slice content
        assert_eq!(partial_slice.as_str(), "line 2\nline 3\nline 4");
        assert_eq!(partial_slice.get(0).unwrap().as_str(), "line 2");
        assert_eq!(partial_slice.get(1).unwrap().as_str(), "line 3");
        assert_eq!(partial_slice.get(2).unwrap().as_str(), "line 4");

        // Test empty slice
        let empty_slice = buffer.slice(2..2);
        assert_eq!(empty_slice.len(), 0);

        // Test out of bounds slice
        let out_of_bounds = buffer.slice(4..10);
        assert_eq!(out_of_bounds.len(), 1);
        assert_eq!(out_of_bounds.get(0).unwrap().as_str(), "line 5");
    }

    #[test]
    fn test_slice_of_slice() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = LogBuf::new(content);

        // Create first slice
        let first_slice = buffer.slice(1..4); // lines 2-4
        assert_eq!(first_slice.len(), 3);
        assert_eq!(first_slice.get(0).unwrap().as_str(), "line 2");
        assert_eq!(first_slice.get(1).unwrap().as_str(), "line 3");
        assert_eq!(first_slice.get(2).unwrap().as_str(), "line 4");

        // Create slice of the first slice
        let nested_slice = first_slice.slice(1..3); // lines 3-4
        println!("nested: {} - {}", nested_slice.start, nested_slice.end);
        assert_eq!(nested_slice.len(), 2);
        assert_eq!(nested_slice.get(0).unwrap().as_str(), "line 3");
        assert_eq!(nested_slice.get(1).unwrap().as_str(), "line 4");

        // Test that the original slices are unaffected
        assert_eq!(first_slice.len(), 3);
        assert_eq!(buffer.len(), 5);

        // Test empty nested slice
        let empty_nested = first_slice.slice(1..1);
        assert_eq!(empty_nested.len(), 0);
        assert!(empty_nested.is_empty());
    }

    #[test]
    fn test_out_of_range_slices() {
        let content = "line 1\nline 2\nline 3".to_string();
        let buffer = LogBuf::new(content);

        // Test completely out of range
        let out_of_range = buffer.slice(10..15);
        assert_eq!(out_of_range.len(), 0);
        assert!(out_of_range.get(0).is_none());
        assert!(out_of_range.is_empty());

        // Test partially out of range
        let partially_out = buffer.slice(1..10);
        assert_eq!(partially_out.len(), 2);
        assert_eq!(partially_out.get(0).unwrap().as_str(), "line 2");
        assert_eq!(partially_out.get(1).unwrap().as_str(), "line 3");
        assert!(partially_out.get(2).is_none());
    }

    #[test]
    fn test_slice_with_out_of_range_index() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = LogBuf::new(content);

        // Create a slice of just lines 2-3
        let slice = buffer.slice(1..3);
        assert_eq!(slice.len(), 2);
        assert_eq!(slice.get(0).unwrap().as_str(), "line 2");
        assert_eq!(slice.get(1).unwrap().as_str(), "line 3");

        // Try to access line 4 (index 3 in original buffer, but out of range in the slice)
        assert!(slice.get(2).is_none());

        // Try to access line 5 (index 4 in original buffer, but out of range in the slice)
        assert!(slice.get(3).is_none());
    }
}
