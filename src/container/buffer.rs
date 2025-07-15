use std::ops::{Deref, Range};

use rayon::prelude::*;

use super::{arc_str::ArcStr, line_index::LineIndex};

/// A cheap-to-clone container for storage and retrieval of log lines.
///
/// `Buffer` stores the full log content as a single string, and maintains
/// an index of line endings for quick access to individual lines. The line
/// indices are created in parallel when handling large strings.
///
/// # Examples
///
/// ```
/// use analogz::container::Buffer;
///
/// let logs = Buffer::new("line 1\nline 2\nline 3".to_string());
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
pub struct Buffer {
    astr: ArcStr,
    index: LineIndex,
}

impl Buffer {
    /// Creates a new `Buffer` from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The string content to be stored in the buffer
    ///
    /// # Returns
    ///
    /// A new `Buffer` instance containing the provided content
    ///
    pub fn new(content: String) -> Buffer {
        Buffer {
            index: LineIndex::build(&content),
            astr: ArcStr::from(content),
        }
    }

    /// Returns the underlying string content as `&str`.
    ///
    /// # Returns
    ///
    /// A `&str` containing the entire log content
    pub fn as_str(&self) -> &str {
        if self.index.is_empty() {
            &self.astr
        } else {
            let start = self.index.line_start(0).unwrap();
            let end = self.index.line_end(self.index.len() - 1).unwrap();
            &self.astr[start..end]
        }
    }

    /// Returns the number of lines in the log buffer.
    ///
    /// # Returns
    ///
    /// The total number of lines in the log buffer
    pub fn len(&self) -> usize {
        self.index.len()
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
        let start = self.index.line_start(idx)?;
        let end = self.index.line_end(idx)?;
        Some(Line {
            astr: self.astr.slice(start..end),
        })
    }

    /// Returns a slice of the log buffer for the given range of lines.
    ///
    /// # Arguments
    ///
    /// * `rng` - A range of line indices to include in the slice
    ///
    /// # Returns
    ///
    /// A new `Buffer` containing only the lines in the specified range
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::container::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3\nline 4".to_string());
    /// let middle_lines = logs.slice(1..3);
    /// assert_eq!(middle_lines.len(), 2);
    /// assert_eq!(middle_lines.get(0).unwrap().as_str(), "line 2");
    /// assert_eq!(middle_lines.get(1).unwrap().as_str(), "line 3");
    /// ```
    pub fn slice(&self, rng: Range<usize>) -> Buffer {
        Self {
            astr: self.astr.clone(),
            index: self.index.slice(rng.start..rng.end + 1),
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
    /// use analogz::container::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3".to_string());
    ///
    /// for line in logs.iter() {
    ///     println!("{}", line.as_str());
    /// }
    /// ```
    pub fn iter(&self) -> LineIter {
        LineIter {
            buffer: self.clone(),
            idx: 0,
        }
    }
}

/// Iterator over the lines in a `Buffer`.
///
/// Created by the `Buffer::iter()` or `Buffer::iter_from()` methods.
#[derive(Debug)]
pub struct LineIter {
    buffer: Buffer,
    idx: usize,
}

impl Iterator for LineIter {
    type Item = Line;

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
pub struct Line {
    astr: ArcStr,
}

impl Line {
    pub fn start(&self) -> usize {
        self.astr.start()
    }

    pub fn end(&self) -> usize {
        self.astr.end()
    }
}

impl Deref for Line {
    type Target = ArcStr;

    fn deref(&self) -> &Self::Target {
        &self.astr
    }
}

impl From<Line> for ArcStr {
    fn from(value: Line) -> Self {
        value.astr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buffer = Buffer::new(String::new());
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.as_str(), "");
        assert!(buffer.get(0).is_none());
        assert_eq!(buffer.iter().count(), 0);
    }

    #[test]
    fn test_single_line() {
        let content = "single line".to_string();
        let buffer = Buffer::new(content.clone());

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
        let buffer = Buffer::new(content.clone());

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
        let buffer = Buffer::new(content);
        let line = buffer.get(0).unwrap();

        // Test AsRef<str> implementation
        let str_ref: &str = line.as_ref();
        assert_eq!(str_ref, "test line");
    }

    #[test]
    fn test_trailing_newline() {
        let content = "line 1\nline 2\n".to_string();
        let buffer = Buffer::new(content);

        assert_eq!(buffer.len(), 3); // Two explicit lines plus empty line at end

        let lines: Vec<_> = buffer.iter().map(|l| l.as_str().to_string()).collect();
        assert_eq!(lines, vec!["line 1", "line 2", ""]);
    }

    #[test]
    fn test_consecutive_newlines() {
        let content = "line 1\n\nline 3".to_string();
        let buffer = Buffer::new(content);

        assert_eq!(buffer.len(), 3);

        let lines: Vec<_> = buffer.iter().map(|l| l.as_str().to_string()).collect();
        assert_eq!(lines, vec!["line 1", "", "line 3"]);
    }

    #[test]
    fn test_large_content() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("Line number {i}\n"));
        }

        let buffer = Buffer::new(content.clone());
        assert_eq!(buffer.len(), 1001); // 1000 lines + empty line at end

        // Check random lines
        let line42 = buffer.get(42).unwrap();
        assert_eq!(line42.as_str(), "Line number 42");

        let line999 = buffer.get(999).unwrap();
        assert_eq!(line999.as_str(), "Line number 999");
    }

    #[test]
    fn test_slice() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

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
        let buffer = Buffer::new(content);

        // Create first slice
        let first_slice = buffer.slice(1..4); // lines 2-4
        assert_eq!(first_slice.len(), 3);
        assert_eq!(first_slice.get(0).unwrap().as_str(), "line 2");
        assert_eq!(first_slice.get(1).unwrap().as_str(), "line 3");
        assert_eq!(first_slice.get(2).unwrap().as_str(), "line 4");

        // Create slice of the first slice
        let nested_slice = first_slice.slice(1..3); // lines 3-4
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
        let buffer = Buffer::new(content);

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
        let buffer = Buffer::new(content);

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
