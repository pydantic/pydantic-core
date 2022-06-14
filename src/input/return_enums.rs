use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub enum EitherBytes<'a> {
    Rust(Vec<u8>),
    Python(&'a PyBytes),
}

impl<'a> EitherBytes<'a> {
    pub fn into_pybytes(self, py: Python<'a>) -> &'a PyBytes {
        match self {
            EitherBytes::Rust(bytes) => PyBytes::new(py, &bytes),
            EitherBytes::Python(py_bytes) => py_bytes,
        }
    }

    pub fn len(&'a self) -> PyResult<usize> {
        match self {
            EitherBytes::Rust(bytes) => Ok(bytes.len()),
            EitherBytes::Python(py_bytes) => py_bytes.len(),
        }
    }
}
