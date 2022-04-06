use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Layer {
    function: PyObject,
    next_layer: Option<Box<Layer>>,
}

impl Layer {
    pub fn new(py: Python, function: PyObject, next_layer: Option<Box<Self>>) -> PyResult<Self> {
        let f_any: &PyAny = function.extract(py)?;
        if !f_any.is_callable() {
            return Err(PyTypeError::new_err(format!("{:?} is not callable", f_any)));
        }
        Ok(Self { function, next_layer })
    }
}

#[pymethods]
impl Layer {
    fn __call__(&self, py: Python, arg: PyObject) -> PyResult<PyObject> {
        if let Some(next_layer) = &self.next_layer {
            let layer = *next_layer.clone();
            self.function.call(py, (arg, layer), None)
        } else {
            self.function.call1(py, (arg,))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("OnionLayer({:?})", self))
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Onion {
    layer: Layer,
}

#[pymethods]
impl Onion {
    #[new]
    fn py_new(py: Python, mut functions: Vec<PyObject>) -> PyResult<Self> {
        let innermost = functions.pop().ok_or_else(|| PyValueError::new_err("Empty onion"))?;
        let mut layer = Layer::new(py, innermost, None)?;
        functions.reverse();
        for function in functions {
            layer = Layer::new(py, function, Some(Box::new(layer)))?;
        }
        Ok(Self { layer })
    }

    fn __call__(&self, py: Python, arg: PyObject) -> PyResult<PyObject> {
        self.layer.__call__(py, arg)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Onion({:?})", self))
    }
}
