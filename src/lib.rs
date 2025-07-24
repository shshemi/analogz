mod container;

use polars::frame::DataFrame;
use pyo3::prelude::*;
use regex::Regex;

use crate::container::{ArcStr, Buffer, LineIter};

#[pymodule]
fn _lib_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBuffer>()?;
    m.add_class::<PyLineIter>()?;
    m.add_class::<PyArcStr>()?;
    m.add_class::<PyCompiledRegex>()?;
    Ok(())
}

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

    pub fn get(&self, idx: usize) -> PyResult<PyArcStr> {
        self.buffer
            .get(idx)
            .map(Into::into)
            .map(PyArcStr)
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

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.buffer.as_str().to_owned()
    }
}

#[pyclass]
pub struct PyLineIter(LineIter);

#[pymethods]
impl PyLineIter {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<PyArcStr> {
        self.0.next().map(Into::into).map(PyArcStr)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyArcStr(ArcStr);

#[pymethods]
impl PyArcStr {
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
#[derive(Debug, Clone)]
pub struct PyCompiledRegex(Regex);

#[pymethods]
impl PyCompiledRegex {
    #[new]
    pub fn new(re: String) -> PyResult<Self> {
        Ok(Self(Regex::new(&re).map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("invlid pattern")
        })?))
    }

    pub fn find(&self, context: PyArcStr) -> Option<PyArcStr> {
        self.0
            .find(context.0.as_str())
            .map(|m| context.0.slice(m.start()..m.end()))
            .map(PyArcStr)
    }
}

impl PyCompiledRegex {
    pub fn into_inner(self) -> Regex {
        self.0
    }
}
