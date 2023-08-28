use pyo3::exceptions::{PyAttributeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

use ahash::AHashMap;

#[derive(Debug, Clone)]
#[pyclass]
pub struct RustModel {
    parent: PyObject,
    data: Vec<PyObject>,
}

#[pymethods]
impl RustModel {
    fn __getattr__(&self, py: Python, key: &PyString) -> PyResult<PyObject> {
        let key_str = key.to_str()?;
        let parent: ModelByIter = self.parent.extract(py)?;
        if let Some(index) = parent.field_lookup.get(key_str) {
            Ok(self.data[*index].clone_ref(py))
        } else {
            Err(PyAttributeError::new_err(format!(
                "Field {} not found",
                key.to_string_lossy()
            )))
        }
    }

    fn model_dump(&self, py: Python) -> PyResult<PyObject> {
        let parent: ModelByIter = self.parent.extract(py)?;
        let output = PyDict::new(py);
        for (key, value) in parent.field_names.iter().zip(self.data.iter()) {
            output.set_item(key, value)?;
        }
        Ok(output.into_py(py))
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct ModelByIter {
    field_lookup: AHashMap<String, usize>,
    field_names: Vec<Py<PyString>>,
    scratch: Vec<Option<PyObject>>,
}

#[pymethods]
impl ModelByIter {
    #[new]
    fn py_new(py: Python, raw_fields: &PyList) -> PyResult<Self> {
        let capacity = raw_fields.len();
        let mut field_lookup = AHashMap::with_capacity(capacity);
        let mut field_names = Vec::with_capacity(capacity);
        for (index, raw_field) in raw_fields.iter().enumerate() {
            let f_py: &PyString = raw_field.downcast()?;
            field_names.push(f_py.into_py(py));
            field_lookup.insert(f_py.to_str()?.to_string(), index);
        }
        let scratch = vec![None; capacity];
        Ok(Self {
            field_lookup,
            field_names,
            scratch,
        })
    }

    fn validate(slf: &PyCell<Self>, py: Python, data: &PyAny) -> PyResult<RustModel> {
        let mut py_ref_mut_self = slf.borrow_mut();
        let self_ = &mut *py_ref_mut_self;

        let dict: &PyDict = data.downcast()?;
        for (k, v) in dict {
            let f = k.downcast::<PyString>()?.to_str()?;
            if let Some(index) = self_.field_lookup.get(f) {
                let elem = unsafe { self_.scratch.get_unchecked_mut(*index) };
                *elem = Some(v.to_object(py));
                // self_.scratch[*index] = Some(v.to_object(py));
            } else {
                return Err(PyValueError::new_err(format!("Key `{f}` not expected")));
            }
        }
        let mut output_data = Vec::with_capacity(self_.scratch.len());
        for (index, r) in self_.scratch.iter_mut().enumerate() {
            if let Some(field_data) = r.take() {
                output_data.push(field_data);
            } else {
                let name = self_.field_names[index].as_ref(py).to_str()?;
                return Err(PyValueError::new_err(format!("Iter: Field `{name}` not fill")));
            }
        }
        Ok(RustModel {
            data: output_data,
            parent: slf.to_object(py),
        })
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct ModelByLookup {
    fields: Vec<Py<PyString>>,
}

#[pymethods]
impl ModelByLookup {
    #[new]
    fn py_new(py: Python, raw_fields: &PyList) -> PyResult<Self> {
        let mut fields = Vec::new();
        for raw_field in raw_fields.iter() {
            let f: &PyString = raw_field.downcast()?;
            fields.push(f.into_py(py));
        }
        Ok(Self { fields })
    }

    fn validate(&self, py: Python, data: &PyAny) -> PyResult<Py<PyDict>> {
        let dict: &PyDict = data.downcast()?;
        let output = PyDict::new(py);
        for field_ob in &self.fields {
            let field = field_ob.as_ref(py);
            if let Some(v) = dict.get_item(field) {
                output.set_item(field, v)?;
            } else {
                let field_name = field.to_str()?;
                return Err(PyValueError::new_err(format!("Lookup: Field {field_name} not filled",)));
            }
        }
        Ok(output.into_py(py))
    }
}
