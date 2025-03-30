use std::{ops::Deref, sync::Arc};

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
/// use analogz::buf::LogBuf;
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
        let lines = if content.is_empty() {
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
            lines,
        }
    }

    /// Returns the underlying string content as `&str`.
    ///
    /// # Returns
    ///
    /// A `&str` containing the entire log content
    pub fn as_str(&self) -> &str {
        self.buffer.deref()
    }

    /// Returns the number of lines in the log buffer.
    ///
    /// # Returns
    ///
    /// The total number of lines in the log buffer
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Checks if the log buffer is empty (contains no lines).
    ///
    /// # Returns
    ///
    /// `true` if the log buffer contains no lines, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        Some(if idx == 0 {
            let end = *self.lines.first()?;
            Line {
                slice: &self.buffer[..end],
                start: 0,
                end,
            }
        } else {
            let start = self.lines.get(idx - 1)? + 1;
            let end = *self.lines.get(idx)?;
            Line {
                slice: &self.buffer[start..end],
                start,
                end,
            }
        })
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
    /// use analogz::buf::LogBuf;
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

    /// Returns an iterator over lines starting from the given index.
    ///
    /// This is similar to `iter()`, but allows specifying a starting point
    /// in the log buffer.
    ///
    /// # Arguments
    ///
    /// * `idx` - The zero-based index to start iterating from
    ///
    /// # Returns
    ///
    /// An iterator that yields each line starting from the given index
    pub fn iter_from(&self, idx: usize) -> LineIter {
        LineIter { buffer: self, idx }
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

impl AsRef<str> for Line<'_> {
    fn as_ref(&self) -> &str {
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
    fn test_iter_lines_from() {
        let content = "line 1\nline 2\nline 3\nline 4".to_string();
        let buffer = LogBuf::new(content);

        let lines: Vec<_> = buffer
            .iter_from(1)
            .map(|l| l.as_str().to_string())
            .collect();
        assert_eq!(lines, vec!["line 2", "line 3", "line 4"]);

        let lines: Vec<_> = buffer
            .iter_from(2)
            .map(|l| l.as_str().to_string())
            .collect();
        assert_eq!(lines, vec!["line 3", "line 4"]);

        // Start from the end
        let lines: Vec<_> = buffer.iter_from(4).collect();
        assert!(lines.is_empty());
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
}
