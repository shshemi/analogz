use pyo3::prelude::*;

#[pymodule]
fn _lib_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
