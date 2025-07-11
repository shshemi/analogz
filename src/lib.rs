mod container;

use pyo3::prelude::*;

use crate::container::{Buffer, Line, LineIter};

#[pyclass]
pub struct PyBuffer(Buffer);

#[pymethods]
impl PyBuffer {
    #[new]
    /// Create a new PyBuffer from the given string content.
    pub fn new(content: String) -> Self {
        PyBuffer(Buffer::new(content))
    }

    pub fn get(&self, idx: usize) -> PyResult<PyLine> {
        self.0
            .get(idx)
            .map(PyLine)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyIndexError, _>("index out of range"))
    }

    pub fn slice(&self, start: Option<usize>, end: Option<usize>) -> Self {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(self.len());
        PyBuffer(self.0.slice(start..end))
    }

    pub fn iter(&self) -> PyLineIter {
        PyLineIter(self.0.iter())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[pyclass]
pub struct PyLineIter(LineIter);

#[pymethods]
impl PyLineIter {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<PyLine> {
        self.0.next().map(PyLine)
    }
}

#[pyclass]
pub struct PyLine(Line);

#[pymethods]
impl PyLine {
    pub fn start(&self) -> usize {
        self.0.start()
    }

    pub fn end(&self) -> usize {
        self.0.end()
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[pymodule]
fn _lib_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBuffer>()?;
    m.add_class::<PyLineIter>()?;
    m.add_class::<PyLine>()?;
    Ok(())
}
