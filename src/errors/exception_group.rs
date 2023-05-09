use pyo3::exceptions::{PyException, PyNotImplementedError, PyTypeError, PyValueError, PyAssertionError};
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{intern, prelude::*, AsPyPointer};

use crate::errors::PydanticException;

#[derive(Debug, Clone, FromPyObject)]
enum LocItem{
    String(String),
    Index(usize),
}

impl IntoPy<PyObject> for LocItem {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            LocItem::String(s) => s.into_py(py),
            LocItem::Index(i) => i.into_py(py),
        }
    }
}

type SplitResult<T> = (Option<T>, Option<T>);

#[derive(Debug, Clone)]
struct PyExceptionGroup(Py<PyAny>);

impl<'source> FromPyObject<'source> for PyExceptionGroup {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        // TODO: simplify after https://github.com/PyO3/pyo3/pull/3141
        let tp = ob.get_type();
        if tp.is_subclass_of::<PyException>()? {
            let name = tp.name()?;
            if name == "BaseExceptionGroup" || name == "ExceptionGroup" {
                return Ok(PyExceptionGroup(ob.into()));
            }
        }
        Err(PyTypeError::new_err(""))
    }
}

impl PyExceptionGroup {
    pub fn compare(&self, py: Python, other: &Self) -> PyResult<bool> {
        self.0.as_ref(py).eq(other.0.as_ref(py))
    }
    pub fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyAny) -> PyResult<bool>,
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

    pub fn flatten_errors(&self, py: Python) -> PyResult<Vec<LocAndError>> {
        let exceptions: Vec<InnerException> = self.0.getattr(py, intern!(py, "exceptions"))?.extract(py)?;
        flatten_errors(py, &vec![], &exceptions)
    }
}

impl IntoPy<PyObject> for PyExceptionGroup {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0.into_py(py)
    }
}

#[derive(Debug, Clone, FromPyObject)]
enum AssertionOrValueError {
    ValueError(Py<PyException>),
    AssertionError(Py<PyException>),
}

impl AssertionOrValueError {
    pub fn eq(&self, other: &Self) -> PyResult<bool> {
        let cmp = |l: Py<PyException>, r: Py<PyException>| {
            if l.as_ptr() == r.as_ptr() {
                Ok(true)
            } else {
                l.as_ref(py).eq(r.as_ref(py))
            }
        };
        match (self, other) {
            (AssertionOrValueError::ValueError(l), AssertionOrValueError::ValueError(r)) => cmp(l, r),
            (AssertionOrValueError::ValueError(_), AssertionOrValueError::AssertionError(_)) => Ok(false),
            (AssertionOrValueError::AssertionError(_), AssertionOrValueError::ValueError(_)) => Ok(false),
            (AssertionOrValueError::AssertionError(l), AssertionOrValueError::AssertionError(r)) => cmp(l, r),
        }
    }

    pub fn as_ref(&self, py: Python) -> &PyAny {
        match self {
            AssertionOrValueError::ValueError(e) => e.as_ref(py),
            AssertionOrValueError::AssertionError(e) => e.as_ref(py),
        }
    }
}


// Wrap the exceptions we contain
// Determine up front what type of exception they are to make
// life easier when we subsequently have to iterate through them
// Keep leaf exceptions wrapped in a `Py` since for the most part we just
// hold references to them and pass them back and forth to Python,
// we don't interact with them all that much
#[derive(Debug, Clone)]
enum InnerException {
    ValidationException(Py<ValidationException>),
    PyExceptionGroup(PyExceptionGroup),
    PydanticException(Py<PyAny>),
    AssertionOrValueError(AssertionOrValueError),  // Always a ValueError or AssertionError
}

impl<'source> FromPyObject<'source> for InnerException {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let py = ob.py();
        if let Ok(v) = ob.extract::<ValidationException>() {
            Ok(InnerException::ValidationException(Py::new(py, v)?))
        } else if let Ok(_) = ob.extract::<PydanticException>() {
            Ok(InnerException::PydanticException(ob.into()))
        } else if let Ok(v) = ob.extract::<PyExceptionGroup>() {
            Ok(InnerException::PyExceptionGroup(PyExceptionGroup(v.into_py(py))))
        } else if let Ok(_) = ob.downcast::<PyValueError>() {
            Ok(InnerException::AssertionOrValueError(AssertionOrValueError::ValueError(ob.into_py(py))))
        } else if let Ok(_) = ob.downcast::<PyAssertionError>() {
            Ok(InnerException::AssertionOrValueError(AssertionOrValueError::ValueError(ob.into_py(py))))
        } else {
            Err(PyTypeError::new_err(""))
        }
    }
}

impl InnerException {
    /// Like ==, but requires a py token
    pub fn compare(&self, py: Python, other: &Self) -> PyResult<bool> {
        match (self, other) {
            (InnerException::ValidationException(l), InnerException::ValidationException(r)) => {
                let l = l.try_borrow(py)?;
                let r = r.try_borrow(py)?;
                l.compare(py, &r)
            }
            (InnerException::ValidationException(_), InnerException::PyExceptionGroup(_)) => Ok(false),
            (InnerException::ValidationException(_), InnerException::PydanticException(_)) => Ok(false),
            (InnerException::ValidationException(_), InnerException::AssertionOrValueError(_)) => Ok(false),

            (InnerException::PyExceptionGroup(_), InnerException::ValidationException(_)) => Ok(false),
            (InnerException::PyExceptionGroup(l), InnerException::PyExceptionGroup(r)) => l.compare(py, r),
            (InnerException::PyExceptionGroup(_), InnerException::PydanticException(_)) => Ok(false),
            (InnerException::PyExceptionGroup(_), InnerException::AssertionOrValueError(_)) => Ok(false),

            (InnerException::AssertionOrValueError(_), InnerException::ValidationException(_)) => Ok(false),
            (InnerException::AssertionOrValueError(_), InnerException::PyExceptionGroup(_)) => Ok(false),
            (InnerException::AssertionOrValueError(_), InnerException::PydanticException(_)) => Ok(false),
            (InnerException::AssertionOrValueError(l), InnerException::AssertionOrValueError(r)) => l.eq(r),

            (InnerException::PydanticException(_), InnerException::ValidationException(_)) => Ok(false),
            (InnerException::PydanticException(_), InnerException::PyExceptionGroup(_)) => Ok(false),
            (InnerException::PydanticException(l), InnerException::PydanticException(r)) => {
                // We should be able to do this in Rust, but it's been a total pain to pass
                // traits through the PydanticException Enum
                // Comparing errors shouldn't be in a performance critical path anyway, so this will do for now
                l.as_ref(py).eq(r)
            }
            (InnerException::PydanticException(_), InnerException::AssertionOrValueError(_)) => Ok(false),
        }
    }

    fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyAny) -> PyResult<bool>,
    ) -> PyResult<SplitResult<Self>> {
        match self {
            InnerException::AssertionOrValueError(exc) => {
                if condition(exc.as_ref(py))? {
                    Ok((Some(InnerException::AssertionOrValueError(exc.clone())), None))
                } else {
                    Ok((None, Some(InnerException::AssertionOrValueError(exc.clone()))))
                }
            }
            InnerException::PydanticException(exc) => {
                if condition(exc.as_ref(py))? {
                    Ok((Some(InnerException::PydanticException(exc.clone())), None))
                } else {
                    Ok((None, Some(InnerException::PydanticException(exc.clone()))))
                }
            }
            InnerException::ValidationException(eg) => {
                let rf = eg.try_borrow(py)?;
                let res = rf.split(py, condition)?;
                let k = match res.0 {
                    Some(v) => Some(InnerException::ValidationException(Py::new(py, v)?)),
                    None => None,
                };
                let d = match res.1 {
                    Some(v) => Some(InnerException::ValidationException(Py::new(py, v)?)),
                    None => None,
                };
                Ok((k, d))
            }
            InnerException::PyExceptionGroup(eg) => {
                let res = eg.split(py, condition)?;
                Ok((
                    res.0.map(InnerException::PyExceptionGroup),
                    res.1.map(InnerException::PyExceptionGroup),
                ))
            }
        }
    }
}

impl IntoPy<PyObject> for InnerException {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            InnerException::ValidationException(eg) => eg.into_py(py),
            InnerException::AssertionOrValueError(exc) => exc.into_py(py),
            InnerException::PydanticException(exc) => exc.into_py(py),
            InnerException::PyExceptionGroup(exc) => exc.into_py(py),
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(extends = PyException, unsendable, module = "pydantic_core._pydantic_core")] // TODO: why is PyException unsendable?
pub struct ValidationException {
    // Our fields
    #[pyo3(get)]
    loc: Vec<LocItem>,

    // General exception fields, mirrors Python 3.11+'s ExceptionGroup
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

type LocAndError = (Vec<LocItem>, InnerException);

impl ValidationException {
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

    fn flatten_errors(&self, py: Python) -> PyResult<Vec<LocAndError>> {
        flatten_errors(py, &self.loc, &self.exceptions)
    }

    fn split(
        &self,
        py: Python,
        condition: &mut impl FnMut(&PyAny) -> PyResult<bool>,
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
            false => Some(
                Self {
                    loc: self.loc.clone(),
                    message: self.message.clone(),
                    exceptions: keep,
                    notes: self.notes.clone(),
                    cause: self.cause.clone(),
                    context: self.context.clone(),
                    traceback: self.traceback.clone(),
                }
            ),
            true => None,
        };
        let self_discard = match discard.is_empty() {
            false => Some(Self {
                loc: self.loc.clone(),
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
impl ValidationException {
    #[new]
    #[pyo3(signature = (message, exceptions, loc = None, *_args, **_kwargs))]
    fn py_new(
        py: Python,
        message: String,
        exceptions: Vec<InnerException>,
        loc: Option<Vec<LocItem>>,
        _args: &PyTuple,
        _kwargs: Option<&PyDict>,
    ) -> PyResult<Self> {
        if exceptions.is_empty() {
            return Err(PyValueError::new_err(
                "second argument (exceptions) must be a non-empty sequence",
            ));
        }
        let notes: &PyList = PyList::empty(py);
        Ok(Self {
            loc: loc.unwrap_or_default(),
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
        let mut condition = |exc: &PyAny| py_condition.call1((exc,))?.extract();
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
        let excs: String = self.exceptions.clone().into_py(py).as_ref(py).repr()?.extract()?;
        let loc: String = self.loc.clone().into_py(py).as_ref(py).repr()?.extract()?;
        Ok(format!("ValidationException(\"{message}\", {excs}, loc={loc})"))
    }

    fn errors(&self, py: Python) -> PyResult<Vec<Py<PyDict>>> {
        let excs = self.flatten_errors(py)?;
        let mut res = Vec::with_capacity(excs.len());
        for (loc, exc) in excs.into_iter() {
            match exc {
                InnerException::PydanticException(e) => {
                    let v: PydanticException = e.extract(py)?;
                    res.push(v.as_py_dict(py)?);
                },
                InnerException::AssertionOrValueError(e) => {
                    let d = PyDict::new(py);
                    let err = e.as_ref(py);
                    let mut message: String;
                    if err.is_instance_of::<PyValueError>(py) {
                        py_err_string!(err.value(py), AssertionError)
                    } else if err.is_instance_of::<PyAssertionError>(py) {
                        py_err_string!(err.value(py), AssertionError)                
                    } else {
                        ValError::InternalErr(err)
                    }
                },
                _ => unreachable!(),
            }
        }
        Ok(res)
    }
}


fn flatten_errors(py: Python, loc: &Vec<LocItem>, exceptions: &Vec<InnerException>) -> PyResult<Vec<LocAndError>> {
    let mut res = vec![];
    let with_loc = |(l, v): (Vec<LocItem>, _)| {
        (loc.iter().cloned().chain(l).collect(), v)
    };
    for e in exceptions {
        match e {
            InnerException::ValidationException(v) => {
                let r = v.as_ref(py);
                let new_excs = v.borrow(py).flatten_errors(py)?;
                let new_excs = new_excs.into_iter().map(with_loc);
                res.extend(new_excs)
            },
            InnerException::PyExceptionGroup(eg) => {
                let new_excs = eg.flatten_errors(py)?;
                let new_excs = new_excs.into_iter().map(with_loc);
                res.extend(new_excs)
            }
            InnerException::PydanticException(_) | InnerException::AssertionOrValueError(_) => res.push((loc.clone(), e.clone())),
        }
    }
    Ok(res)
}