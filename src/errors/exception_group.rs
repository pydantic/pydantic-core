use pyo3::exceptions::{PyBaseException, PyNotImplementedError, PyTypeError, PyValueError};
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{intern, prelude::*, AsPyPointer};

type SplitResult<T> = (Option<T>, Option<T>);

#[derive(Debug, Clone)]
pub struct PyBaseExceptionGroup(Py<PyAny>);

impl<'source> FromPyObject<'source> for PyBaseExceptionGroup {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        // TODO: simplify after https://github.com/PyO3/pyo3/pull/3141
        let tp = ob.get_type();
        if tp.is_subclass_of::<PyBaseException>()? {
            let name = tp.name()?;
            if name == "BaseExceptionGroup" || name == "ExceptionGroup" {
                return Ok(PyBaseExceptionGroup(ob.into()));
            }
        }
        Err(PyTypeError::new_err(""))
    }
}

impl PyBaseExceptionGroup {
    pub fn compare(&self, py: Python, other: &Self) -> PyResult<bool> {
        self.0.as_ref(py).eq(other.0.as_ref(py))
    }
    pub fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyBaseException) -> PyResult<bool>,
    ) -> PyResult<SplitResult<Self>> {
        let mut keep: Vec<Py<PyAny>> = vec![];
        let mut discard: Vec<Py<PyAny>> = vec![];
        let exceptions: Vec<InnerException> = self.0.getattr(py, intern!(py, "exceptions"))?.extract(py)?;
        for inner in exceptions {
            let (maybe_keep, maybe_discard) = inner.split(py, condition)?;
            if let Some(k) = maybe_keep {
                keep.push(k.into_py(py))
            };
            if let Some(d) = maybe_discard {
                discard.push(d.into_py(py))
            };
        }
        let derive = self.0.getattr(py, intern!(py, "derive"))?;
        let self_keep = match keep.is_empty() {
            false => Some(derive.call1(py, (keep,))?.extract(py)?),
            true => None,
        };
        let self_discard = match discard.is_empty() {
            false => Some(derive.call1(py, (discard,))?.extract(py)?),
            true => None,
        };
        Ok((self_keep, self_discard))
    }
}

impl IntoPy<PyObject> for PyBaseExceptionGroup {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0.into_py(py)
    }
}

impl ToPyObject for PyBaseExceptionGroup {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.clone().into_py(py)
    }
}

// Wrap the exceptions we contain
// Determine up front what type of exception they are to make
// life easier when we subsequently have to iterate through them
// Keep leaf exceptions wrapped in a `Py` since for the most part we just
// hold references to them and pass them back and forth to Python,
// we don't interact with them all that much
#[derive(FromPyObject, Debug, Clone)]
pub enum InnerException {
    // Order here matters for FromPyObject!
    #[pyo3(transparent, annotation = "BaseExceptionGroup")]
    BaseExceptionGroup(Py<BaseExceptionGroup>),
    #[pyo3(transparent, annotation = "BaseExceptionGroup")]
    PyBaseExceptionGroup(PyBaseExceptionGroup),
    #[pyo3(transparent, annotation = "BaseException")]
    Leaf(Py<PyBaseException>),
}

impl InnerException {
    /// Like ==, but requires a py token
    pub fn compare(&self, py: Python, other: &Self) -> PyResult<bool> {
        match (self, other) {
            (InnerException::BaseExceptionGroup(l), InnerException::BaseExceptionGroup(r)) => {
                let l = l.try_borrow(py)?;
                let r = r.try_borrow(py)?;
                l.compare(py, &r)
            }
            (InnerException::BaseExceptionGroup(_), InnerException::PyBaseExceptionGroup(_)) => Ok(false),
            (InnerException::BaseExceptionGroup(_), InnerException::Leaf(_)) => Ok(false),

            (InnerException::PyBaseExceptionGroup(_), InnerException::BaseExceptionGroup(_)) => Ok(false),
            (InnerException::PyBaseExceptionGroup(l), InnerException::PyBaseExceptionGroup(r)) => l.compare(py, r),
            (InnerException::PyBaseExceptionGroup(_), InnerException::Leaf(_)) => Ok(false),

            (InnerException::Leaf(_), InnerException::BaseExceptionGroup(_)) => Ok(false),
            (InnerException::Leaf(_), InnerException::PyBaseExceptionGroup(_)) => Ok(false),
            (InnerException::Leaf(l), InnerException::Leaf(r)) => {
                if l.as_ptr() == r.as_ptr() {
                    Ok(true)
                } else {
                    l.as_ref(py).eq(r.as_ref(py))
                }
            }
        }
    }

    fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyBaseException) -> PyResult<bool>,
    ) -> PyResult<SplitResult<Self>> {
        match self {
            InnerException::Leaf(exc) => {
                if condition(exc.as_ref(py))? {
                    Ok((Some(InnerException::Leaf(exc.clone())), None))
                } else {
                    Ok((None, Some(InnerException::Leaf(exc.clone()))))
                }
            }
            InnerException::BaseExceptionGroup(eg) => {
                let rf = eg.try_borrow(py)?;
                let res = rf.split(py, condition)?;
                let k = match res.0 {
                    Some(v) => Some(InnerException::BaseExceptionGroup(Py::new(py, v)?)),
                    None => None,
                };
                let d = match res.1 {
                    Some(v) => Some(InnerException::BaseExceptionGroup(Py::new(py, v)?)),
                    None => None,
                };
                Ok((k, d))
            }
            InnerException::PyBaseExceptionGroup(eg) => {
                let res = eg.split(py, condition)?;
                Ok((
                    res.0.map(InnerException::PyBaseExceptionGroup),
                    res.1.map(InnerException::PyBaseExceptionGroup),
                ))
            }
        }
    }
}

impl IntoPy<PyObject> for InnerException {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            InnerException::BaseExceptionGroup(eg) => eg.into_py(py),
            InnerException::Leaf(exc) => exc.into_py(py),
            InnerException::PyBaseExceptionGroup(exc) => exc.into_py(py),
        }
    }
}

impl ToPyObject for InnerException {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            InnerException::BaseExceptionGroup(eg) => eg.clone().into_py(py),
            InnerException::Leaf(exc) => exc.clone().into_py(py),
            InnerException::PyBaseExceptionGroup(exc) => exc.clone().into_py(py),
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(subclass, extends = PyBaseException, unsendable, module = "pydantic_core._pydantic_core")] // TODO: why is PyBaseException unsendable?
pub struct BaseExceptionGroup {
    // A String, but we never touch it, so keep it in Python
    #[pyo3(get)]
    message: String,
    #[pyo3(get)]
    exceptions: Vec<InnerException>,
    #[pyo3(get, set, name = "__notes__")]
    notes: Py<PyList>,
    #[pyo3(get, set, name = "__cause__")]
    cause: Option<Py<PyAny>>,
    #[pyo3(get, set, name = "__context__")]
    context: Option<Py<PyAny>>,
    #[pyo3(get, set, name = "__traceback__")]
    traceback: Option<Py<PyAny>>,
}

impl BaseExceptionGroup {
    fn compare(&self, py: Python, other: &Self) -> PyResult<bool> {
        Ok(self.message == other.message
            && {
                if self.exceptions.len() != other.exceptions.len() {
                    false
                } else {
                    for (l, r) in self.exceptions.iter().zip(&other.exceptions) {
                        if !(l.compare(py, r)?) {
                            return Ok(false);
                        }
                    }
                    true
                }
            }
            && self.notes.as_ref(py).eq(other.notes.as_ref(py))?)
    }
    pub fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyBaseException) -> PyResult<bool>,
    ) -> PyResult<SplitResult<Self>> {
        let mut keep = vec![];
        let mut discard = vec![];
        for exception in &self.exceptions {
            let (exc_keep, exc_discard) = exception.split(py, condition)?;
            if let Some(exc) = exc_keep {
                keep.push(exc)
            };
            if let Some(exc) = exc_discard {
                discard.push(exc)
            }
        }
        let self_keep = match keep.is_empty() {
            false => Some(Self {
                message: self.message.clone(),
                exceptions: keep,
                notes: self.notes.clone(),
                cause: self.cause.clone(),
                context: self.context.clone(),
                traceback: self.traceback.clone(),
            }),
            true => None,
        };
        let self_discard = match discard.is_empty() {
            false => Some(Self {
                message: self.message.clone(),
                exceptions: discard,
                notes: self.notes.clone(),
                cause: self.cause.clone(),
                context: self.context.clone(),
                traceback: self.traceback.clone(),
            }),
            true => None,
        };
        Ok((self_keep, self_discard))
    }
}

#[pymethods]
impl BaseExceptionGroup {
    #[new]
    #[pyo3(signature = (message, exceptions, *_args, **_kwargs))]
    fn py_new(
        py: Python,
        // Same required arguments as the C version
        message: String, // Just for type checking, we immediately make it a PyAny
        exceptions: Vec<InnerException>,
        _args: &PyTuple,
        _kwargs: Option<&PyDict>,
    ) -> PyResult<Self> {
        if exceptions.is_empty() {
            return Err(PyValueError::new_err(
                "second argument (exceptions) must be a non-empty sequence",
            ));
        }
        let notes: &PyList = PyList::empty(py);
        // TODO: we should get a reference to the current class
        // and construct a subclass if it is one
        Ok(Self {
            message,
            exceptions,
            notes: notes.into(),
            cause: None,
            context: None,
            traceback: None,
        })
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => self.compare(py, other),
            CompareOp::Ne => Ok(!(self.compare(py, other)?)),
            _ => Err(PyNotImplementedError::new_err("")),
        }
    }

    fn subgroup(&self, py: Python, py_condition: &PyAny) -> PyResult<Option<Py<Self>>> {
        let (keep, _) = self.py_split(py, py_condition)?;
        Ok(keep)
    }

    #[pyo3(name = "split")]
    fn py_split(&self, py: Python, py_condition: &PyAny) -> PyResult<SplitResult<Py<Self>>> {
        let mut condition = |exc: &PyBaseException| py_condition.call1((exc,))?.extract();
        let (keep, discard) = self.split(py, &mut condition)?;
        let keep = match keep {
            Some(v) => Some(Py::new(py, v)?),
            None => None,
        };
        let discard = match discard {
            Some(v) => Some(Py::new(py, v)?),
            None => None,
        };
        Ok((keep, discard))
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        let message = &self.message;
        let excs: String = self.exceptions.to_object(py).as_ref(py).repr()?.extract()?;
        Ok(format!("BaseExceptionGroup(\"{message}\", {excs})"))
    }
}
