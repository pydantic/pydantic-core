use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::fmt;

type RustLayerFunction = fn(py: Python, value: &PyAny, next_layer: &OnionLayer) -> PyResult<PyObject>;
type RustCoreFunction = fn(py: Python, value: &PyAny) -> PyResult<PyObject>;

#[derive(Clone)]
pub enum LayerFunction {
    RustLayer(RustLayerFunction, Box<OnionLayer>),
    RustCore(RustCoreFunction),
    PythonLayer(PyObject, Box<OnionLayer>),
    PythonCore(PyObject),
}

impl fmt::Debug for LayerFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let t = match self {
            LayerFunction::RustLayer(_, _) => "RustLayer",
            LayerFunction::RustCore(_) => "RustCore",
            LayerFunction::PythonLayer(_, _) => "PythonLayer",
            LayerFunction::PythonCore(_) => "PythonCore",
        };
        write!(f, "{}", t)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct OnionLayer {
    func: LayerFunction,
}

impl OnionLayer {
    pub fn new_py(py: Python, function: PyObject, next_layer: Option<Box<Self>>) -> PyResult<Self> {
        let f_any: &PyAny = function.extract(py)?;
        if !f_any.is_callable() {
            return Err(PyTypeError::new_err(format!("{:?} is not callable", f_any)));
        }
        if let Some(next_layer) = next_layer {
            Ok(Self {
                func: LayerFunction::PythonLayer(function, next_layer),
            })
        } else {
            Ok(Self {
                func: LayerFunction::PythonCore(function),
            })
        }
    }

    pub fn new_rust(function: RustLayerFunction, next_layer: Box<Self>) -> Self {
        Self {
            func: LayerFunction::RustLayer(function, next_layer),
        }
    }

    pub fn new_rust_core(function: RustCoreFunction) -> Self {
        Self {
            func: LayerFunction::RustCore(function),
        }
    }
}

#[pymethods]
impl OnionLayer {
    fn __call__(&self, py: Python, arg: &PyAny) -> PyResult<PyObject> {
        match &self.func {
            LayerFunction::RustLayer(func, next_layer) => func(py, arg, &*next_layer),
            LayerFunction::RustCore(func) => func(py, arg),
            LayerFunction::PythonLayer(func, next_layer) => {
                let next_layer = *next_layer.clone();
                func.call(py, (arg, next_layer), None)
            }
            LayerFunction::PythonCore(func) => func.call1(py, (arg,)),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("OnionLayer({:?})", self.func))
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Onion {
    layer: OnionLayer,
}

#[pymethods]
impl Onion {
    #[new]
    fn py_new(py: Python, mut functions: Vec<PyObject>) -> PyResult<Self> {
        let mut layer = OnionLayer::new_rust_core(validate_str);
        functions.reverse();
        for function in functions {
            layer = OnionLayer::new_py(py, function, Some(Box::new(layer)))?;
        }
        // layer = OnionLayer::new_rust(prepend, Box::new(layer));
        layer = OnionLayer::new_rust(strip_whitespace, Box::new(layer));
        layer = OnionLayer::new_rust(max_length, Box::new(layer));
        // let innermost = functions.pop().ok_or_else(|| PyValueError::new_err("Empty onion"))?;
        // let mut layer = OnionLayer::new_py(py, innermost, None)?;

        Ok(Self { layer })
    }

    fn __call__(&self, py: Python, arg: &PyAny) -> PyResult<PyObject> {
        self.layer.__call__(py, arg)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

pub fn validate_str(py: Python, v: &PyAny) -> PyResult<PyObject> {
    let s = crate::validators::validate_str(v)?;
    Ok(s.to_object(py))
}

fn max_length(py: Python, value: &PyAny, handler: &OnionLayer) -> PyResult<PyObject> {
    let mut s: String = handler.__call__(value.py(), value)?.extract(py)?;
    s = s.to_uppercase();
    if s.len() > 10 {
        Err(PyValueError::new_err("Too long"))
    } else {
        Ok(s.to_object(py))
    }
}

fn strip_whitespace(py: Python, value: &PyAny, handler: &OnionLayer) -> PyResult<PyObject> {
    let s: String = handler.__call__(value.py(), value)?.extract(py)?;
    Ok(s.trim().to_object(py))
}

#[allow(dead_code)]
fn prepend(py: Python, value: &PyAny, handler: &OnionLayer) -> PyResult<PyObject> {
    let s: String = handler.__call__(py, value)?.extract(py)?;
    Ok(format!("{}{}", "x", s).to_object(py))
}
