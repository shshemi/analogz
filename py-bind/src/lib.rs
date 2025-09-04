use polars::frame::DataFrame;
use pyo3::prelude::*;

use analogz::containers::{ArcStr, Buffer, LineIter, Regex};

#[pymodule]
fn _lib_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBuffer>()?;
    m.add_class::<PyLineIter>()?;
    m.add_class::<PyArcStr>()?;
    m.add_class::<PyRegex>()?;
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

    pub fn map(&self, callable: PyObject) -> Vec<PyObject> {
        Python::with_gil(|py| {
            self.buffer.map(|line| {
                callable
                    .call1(py, (PyArcStr(line.into_arc_str()),))
                    .unwrap()
            })
        })
        .to_vec()
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
    #[new]
    pub fn new(s: String) -> Self {
        PyArcStr(ArcStr::new(s))
    }

    pub fn start(&self) -> usize {
        self.0.start()
    }

    pub fn end(&self) -> usize {
        self.0.end()
    }

    pub fn slice(&self, start: Option<usize>, end: Option<usize>) -> PyResult<Self> {
        let mut itr = self
            .0
            .as_str()
            .char_indices()
            .chain(std::iter::once((self.0.len() + 1, '\0')))
            .enumerate();
        let start = if let Some(start) = start {
            itr.find_map(|(ci, (bi, _))| ci.eq(&start).then_some(bi))
                .ok_or(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "index out of range",
                ))?
        } else {
            0
        };

        let end = if let Some(end) = end {
            itr.find_map(|(ci, (bi, _))| ci.eq(&end).then_some(bi))
                .ok_or(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "index out of range",
                ))?
        } else {
            self.0.len()
        };

        Ok(Self(self.0.slice(start..end)))
    }

    pub fn find(&self, pattern: String) -> Option<PyArcStr> {
        self.0.find(pattern.as_str()).map(PyArcStr)
    }

    pub fn split_at(&self, at: usize) -> (PyArcStr, PyArcStr) {
        let (s1, s2) = self.0.split_at(at);
        (PyArcStr(s1), PyArcStr(s2))
    }

    pub fn contains(&self, other: PyArcStr) -> bool {
        self.0.contains(other.0)
    }

    pub fn boundries(&self) -> (usize, usize) {
        (self.0.start(), self.0.end())
    }

    pub fn rel_position(&self, anchor: PyArcStr) -> Option<isize> {
        anchor.0.relative_position(&self.0)
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn char_count(&self) -> usize {
        self.0.chars().count()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyRegex(Regex);

#[pymethods]
impl PyRegex {
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

impl PyRegex {
    pub fn into_inner(self) -> Regex {
        self.0
    }
}
