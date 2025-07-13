mod container;

use polars::frame::DataFrame;
use pyo3::prelude::*;

use crate::container::{ArcStr, Buffer, Line, LineIter};

#[pyclass]
pub struct PyBuffer {
    buffer: Buffer,
    features: DataFrame,
}

#[pymethods]
impl PyBuffer {
    #[new]
    pub fn new(content: String) -> Self {
        PyBuffer {
            buffer: Buffer::new(content),
            features: DataFrame::empty(),
        }
    }

    pub fn get(&self, idx: usize) -> PyResult<PyLine> {
        self.buffer
            .get(idx)
            .map(PyLine)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyIndexError, _>("index out of range"))
    }

    pub fn slice(&self, start: Option<usize>, end: Option<usize>) -> Self {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(self.len());
        PyBuffer {
            buffer: self.buffer.slice(start..end),
            features: self.features.slice(start as i64, end - start),
        }
    }

    pub fn iter(&self) -> PyLineIter {
        PyLineIter(self.buffer.iter())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.buffer.len()
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

    pub fn find(&self, pattern: String) -> Option<PyArcStr> {
        self.0.find(pattern.as_str()).map(PyArcStr)
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[pyclass]
pub struct PyArcStr(ArcStr);

#[pymethods]
impl PyArcStr {
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
    m.add_class::<PyArcStr>()?;
    Ok(())
}
