use std::ops::{Deref, Range};

use itertools::Itertools;

use crate::{containers::ArcSlice, misc::stepped_range::SteppedRange};

use super::{arc_str::ArcStr, cut_indices::CutIndices};

/// A cheap-to-clone container for storage and retrieval of log lines.
///
/// `Buffer` stores the full log content as a single string, and maintains
/// an index of line endings for quick access to individual lines. The line
/// indices are created in parallel when handling large strings.
///
/// # Examples
///
/// ```
/// use analogz::containers::Buffer;
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
    index: CutIndices,
    select: Option<ArcSlice<usize>>,
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
            index: CutIndices::build_par(&content, |c| c == &b'\n'),
            astr: ArcStr::from(content),
            select: None,
        }
    }

    /// Returns the underlying string content as `&str`.
    ///
    /// # Returns
    ///
    /// A `&str` containing the entire log content
    pub fn as_str(&self) -> &str {
        let start = self.index.start(0).unwrap();
        let end = self.index.end(self.index.len() - 1).unwrap();
        &self.astr[start..end]
    }

    /// Returns the number of lines in the log buffer.
    ///
    /// # Returns
    ///
    /// The total number of lines in the log buffer
    pub fn len(&self) -> usize {
        if let Some(select) = &self.select {
            select.len()
        } else {
            self.index.len()
        }
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
        if let Some(select) = &self.select {
            let idx = select.get(idx).copied()?;
            let start = self.index.start(idx)?;
            let end = self.index.end(idx)?;
            Some(Line {
                astr: self.astr.slice(start..end),
            })
        } else {
            let start = self.index.start(idx)?;
            let end = self.index.end(idx)?;
            Some(Line {
                astr: self.astr.slice(start..end),
            })
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
    /// A new `Buffer` containing only the lines in the specified range
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::containers::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3\nline 4".to_string());
    /// let middle_lines = logs.slice(1..3);
    /// assert_eq!(middle_lines.len(), 2);
    /// assert_eq!(middle_lines.get(0).unwrap().as_str(), "line 2");
    /// assert_eq!(middle_lines.get(1).unwrap().as_str(), "line 3");
    /// ```
    pub fn slice(&self, rng: Range<usize>) -> Buffer {
        if let Some(select) = self.select.clone() {
            Self {
                astr: self.astr.clone(),
                index: self.index.clone(),
                select: Some(select.slice(rng)),
            }
        } else {
            Self {
                astr: self.astr.clone(),
                index: self.index.slice(rng.start..rng.end),
                select: None,
            }
        }
    }

    /// Selects specific lines from the log buffer based on the provided indices.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterable of line indices to include in the new buffer
    ///
    /// # Returns
    ///
    /// A new `Buffer` containing only the selected lines
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::containers::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3\nline 4".to_string());
    /// let selected = logs.select([0, 2]);
    /// assert_eq!(selected.len(), 2);
    /// assert_eq!(selected.get(0).unwrap().as_str(), "line 1");
    /// assert_eq!(selected.get(1).unwrap().as_str(), "line 3");
    /// ```
    pub fn select(&self, items: impl IntoIterator<Item = usize>) -> Buffer {
        if let Some(s) = self.select.clone() {
            Self {
                astr: self.astr.clone(),
                index: self.index.clone(),
                select: Some(s.select(items)),
            }
        } else {
            Self {
                astr: self.astr.clone(),
                index: self.index.clone(),
                select: Some(items.into_iter().filter(|idx| idx < &self.len()).collect()),
            }
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
    /// use analogz::containers::Buffer;
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

    /// Applies a function to each line in the buffer, producing an `ArcSlice`.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure or function that takes a `Line` and returns a value of type `O`.
    ///
    /// # Returns
    ///
    /// An `ArcSlice<O>` containing the results of applying the function to each line.
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::containers::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3".to_string());
    ///
    /// // Map each line to its length
    /// let lengths: Vec<usize> = logs.map(|line| line.as_str().len()).into();
    /// assert_eq!(lengths, vec![6, 6, 6]);
    ///
    /// // Map each line to uppercase
    /// let uppercased: Vec<String> = logs.map(|line| line.as_str().to_uppercase()).into();
    /// assert_eq!(uppercased, vec!["LINE 1", "LINE 2", "LINE 3"]);
    /// ```
    pub fn map<F, O>(&self, f: F) -> ArcSlice<O>
    where
        F: FnMut(Line) -> O,
    {
        self.iter().map(f).collect_vec().into()
    }

    /// Applies a function to each line in the buffer in parallel, producing an `ArcSlice`.
    ///
    /// This method divides the buffer into chunks and processes each chunk in parallel
    /// using multiple threads. The function `f` is applied to each line, and the results
    /// are collected into an `ArcSlice`.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure or function that takes a `Line` and returns a value of type `O`.
    ///
    /// # Returns
    ///
    /// An `ArcSlice<O>` containing the results of applying the function to each line.
    ///
    /// # Examples
    ///
    /// ```
    /// use analogz::containers::Buffer;
    ///
    /// let logs = Buffer::new("line 1\nline 2\nline 3".to_string());
    ///
    /// // Map each line to its length in parallel
    /// let lengths: Vec<usize> = logs.par_map(|line| line.as_str().len()).into();
    /// assert_eq!(lengths, vec![6, 6, 6]);
    ///
    /// // Map each line to uppercase in parallel
    /// let uppercased: Vec<String> = logs.par_map(|line| line.as_str().to_uppercase()).into();
    /// assert_eq!(uppercased, vec!["LINE 1", "LINE 2", "LINE 3"]);
    /// ```
    ///
    /// # Notes
    ///
    /// The order of the results in the `ArcSlice` matches the order of the lines in the buffer.
    /// This method is designed to leverage multiple CPU cores for improved performance on large buffers.
    pub fn par_map<F, O>(&self, f: F) -> ArcSlice<O>
    where
        O: Send,
        F: Fn(Line) -> O + Send + Clone,
    {
        let slice_size = (self.len() / num_cpus::get()).max(1);
        std::thread::scope(|scope| {
            SteppedRange::new(0, self.len(), slice_size)
                .map(|offset| {
                    let f = f.clone();
                    scope.spawn(move || self.slice(offset..offset + slice_size).into_iter().map(f))
                })
                .filter_map(|hndl| hndl.join().ok())
                .flatten()
                .collect_vec()
        })
        .into()
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

impl IntoIterator for Buffer {
    type Item = Line;
    type IntoIter = LineIter;

    fn into_iter(self) -> Self::IntoIter {
        LineIter {
            buffer: self,
            idx: 0,
        }
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

    pub fn into_arc_str(self) -> ArcStr {
        self.astr
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
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.as_str(), "");
        assert_eq!(buffer.get(0).unwrap().as_str(), "");
        assert_eq!(buffer.iter().count(), 1);
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

    #[test]
    fn map_preserves_order_across_chunks() {
        // Build content with many lines to ensure chunking across CPUs
        let mut content = String::new();
        for i in 0..1234 {
            content.push_str(&format!("Line {i}\n"));
        }
        // trailing empty line is included by design
        let buffer = Buffer::new(content);

        // Map each line to its exact string
        let mapped: ArcSlice<Option<String>> =
            buffer.par_map(|line| Some(line.as_str().to_string()));
        let slice: &[Option<String>] = &mapped;

        // Expect len == lines + trailing empty line
        assert_eq!(slice.len(), buffer.len());

        // Spot-check several indices straddle chunk boundaries regardless of CPU count.
        // Check first, a middle, last-1 (before empty), and last (empty).
        assert_eq!(slice.first().unwrap().as_deref(), Some("Line 0"));
        assert_eq!(slice.get(617).unwrap().as_deref(), Some("Line 617"));
        assert_eq!(slice.get(1233).unwrap().as_deref(), Some("Line 1233"));
        assert_eq!(slice.last().unwrap().as_deref(), Some(""));
    }

    #[test]
    fn map_retains_none_entries_without_dropping() {
        let content = "a\nb\nc\nd\ne\n".to_string();
        let buffer = Buffer::new(content);

        let mapped: ArcSlice<Option<&'static str>> =
            buffer.par_map(|line| match line.as_str().chars().next() {
                Some('a') | Some('c') | Some('e') => None,
                Some('b') | Some('d') => Some("ok"),
                _ => Some("empty"),
            });

        let slice: &[Option<&str>] = &mapped;
        assert_eq!(slice.len(), buffer.len());
        assert_eq!(slice[0], None);
        assert_eq!(slice[1], Some("ok"));
        assert_eq!(slice[2], None);
        assert_eq!(slice[3], Some("ok"));
        assert_eq!(slice[4], None);
        assert_eq!(slice[5], Some("empty"));
    }

    #[test]
    fn map_can_use_line_offsets_correctly() {
        let content = "one\n\ntwo\nthree\n".to_string();
        let buffer = Buffer::new(content);

        let mapped: ArcSlice<Option<(usize, usize)>> =
            buffer.par_map(|line| Some((line.start(), line.end())));
        let slice: &[(usize, usize)] = &mapped.iter().map(|o| o.unwrap()).collect::<Vec<_>>();

        assert_eq!(slice, &[(0, 3), (4, 4), (5, 8), (9, 14), (15, 15),]);
    }

    #[test]
    fn map_handles_large_input_correctly() {
        let n = 5000usize;
        let mut content = String::new();
        for i in 0..n {
            content.push_str(&format!("L{i}\n"));
        }
        let buffer = Buffer::new(content);
        let mapped: ArcSlice<Option<usize>> = buffer.par_map(|line| Some(line.as_str().len()));

        let slice: &[Option<usize>] = &mapped;
        assert_eq!(slice.len(), n + 1);

        // Spot checks
        assert_eq!(slice[0], Some("L0".len()));
        assert_eq!(slice[n / 2], Some(format!("L{}", n / 2).len()));
        assert_eq!(slice[n - 1], Some(format!("L{}", n - 1).len()));
        assert_eq!(slice[n], Some(0));
    }

    #[test]
    fn test_select() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

        // Select specific lines
        let selected = buffer.select([0, 2, 4]);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected.get(0).unwrap().as_str(), "line 1");
        assert_eq!(selected.get(1).unwrap().as_str(), "line 3");
        assert_eq!(selected.get(2).unwrap().as_str(), "line 5");

        // Select with repeated indices
        let repeated = buffer.select([1, 1, 3]);
        assert_eq!(repeated.len(), 3);
        assert_eq!(repeated.get(0).unwrap().as_str(), "line 2");
        assert_eq!(repeated.get(1).unwrap().as_str(), "line 2");
        assert_eq!(repeated.get(2).unwrap().as_str(), "line 4");

        // Select with out-of-range indices
        let out_of_range = buffer.select([0, 5, 6]);
        assert_eq!(out_of_range.len(), 1);
        assert_eq!(out_of_range.get(0).unwrap().as_str(), "line 1");
    }

    #[test]
    fn test_slice_then_select() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

        // Slice the buffer
        let sliced = buffer.slice(1..4);
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.get(0).unwrap().as_str(), "line 2");
        assert_eq!(sliced.get(1).unwrap().as_str(), "line 3");
        assert_eq!(sliced.get(2).unwrap().as_str(), "line 4");

        // Select from the sliced buffer
        let selected = sliced.select([0, 2]);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected.get(0).unwrap().as_str(), "line 2");
        assert_eq!(selected.get(1).unwrap().as_str(), "line 4");
    }

    #[test]
    fn test_select_then_slice() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

        // Select specific lines
        let selected = buffer.select([0, 2, 4]);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected.get(0).unwrap().as_str(), "line 1");
        assert_eq!(selected.get(1).unwrap().as_str(), "line 3");
        assert_eq!(selected.get(2).unwrap().as_str(), "line 5");

        // Slice the selected buffer
        let sliced = selected.slice(1..3);
        assert_eq!(sliced.len(), 2);
        assert_eq!(sliced.get(0).unwrap().as_str(), "line 3");
        assert_eq!(sliced.get(1).unwrap().as_str(), "line 5");
    }

    #[test]
    fn test_select_with_empty_result() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

        // Select with no valid indices
        let empty_select = buffer.select([]);
        assert!(empty_select.is_empty());
        assert_eq!(empty_select.len(), 0);

        // Select with out-of-range indices
        let out_of_range = buffer.select([10, 11, 12]);
        assert!(out_of_range.is_empty());
        assert_eq!(out_of_range.len(), 0);
    }

    #[test]
    fn test_slice_and_select_with_empty_result() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5".to_string();
        let buffer = Buffer::new(content);

        // Slice with no valid range
        let empty_slice = buffer.slice(5..5);
        assert!(empty_slice.is_empty());
        assert_eq!(empty_slice.len(), 0);

        // Select from an empty slice
        let empty_select = empty_slice.select([0, 1, 2]);
        assert!(empty_select.is_empty());
        assert_eq!(empty_select.len(), 0);
    }

    #[test]
    fn test_nested_select_and_slice() {
        let content =
            "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10"
                .to_string();
        let buffer = Buffer::new(content);

        // Select specific lines
        let selected = buffer.select([0, 2, 4, 6, 8]);
        assert_eq!(selected.len(), 5);
        assert_eq!(selected.get(0).unwrap().as_str(), "line 1");
        assert_eq!(selected.get(1).unwrap().as_str(), "line 3");
        assert_eq!(selected.get(2).unwrap().as_str(), "line 5");
        assert_eq!(selected.get(3).unwrap().as_str(), "line 7");
        assert_eq!(selected.get(4).unwrap().as_str(), "line 9");

        // Slice the selected buffer
        let sliced = selected.slice(1..4);
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.get(0).unwrap().as_str(), "line 3");
        assert_eq!(sliced.get(1).unwrap().as_str(), "line 5");
        assert_eq!(sliced.get(2).unwrap().as_str(), "line 7");

        // Select again from the sliced buffer
        let nested_select = sliced.select([0, 2]);
        assert_eq!(nested_select.len(), 2);
        assert_eq!(nested_select.get(0).unwrap().as_str(), "line 3");
        assert_eq!(nested_select.get(1).unwrap().as_str(), "line 7");
    }
}
