use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator};

use serde::ser::SerializeSeq;

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::{
    py_err_se_err, BuildSerializer, CombinedSerializer, Extra, ExtraOwned, IncEx, PydanticSerializer, SchemaIncEx,
    SerMode, TypeSerializer,
};

#[derive(Debug, Clone)]
pub struct GeneratorSerializer {
    item_serializer: Box<CombinedSerializer>,
    inc_ex: SchemaIncEx<usize>,
}

impl BuildSerializer for GeneratorSerializer {
    const EXPECTED_TYPE: &'static str = "generator";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let item_serializer = match schema.get_as::<&PyDict>(intern!(py, "items_schema"))? {
            Some(items_schema) => CombinedSerializer::build(items_schema, config, build_context)?,
            None => AnySerializer::build(schema, config, build_context)?,
        };
        Ok(Self {
            item_serializer: Box::new(item_serializer),
            inc_ex: SchemaIncEx::from_schema(schema)?,
        }
        .into())
    }
}

impl GeneratorSerializer {
    fn include_or_exclude<'py>(
        &self,
        py: Python<'py>,
        index: usize,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
    ) -> PyResult<Option<(Option<&'py PyAny>, Option<&'py PyAny>)>> {
        self.inc_ex
            .include_or_exclude(index.to_object(py).as_ref(py), index, include, exclude)
    }
}

impl TypeSerializer for GeneratorSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.iter() {
            Ok(py_iter) => {
                let py = value.py();
                match extra.mode {
                    SerMode::Json => {
                        let item_serializer = self.item_serializer.as_ref();

                        let mut items = match value.len() {
                            Ok(len) => Vec::with_capacity(len),
                            Err(_) => Vec::new(),
                        };
                        for (index, iter_result) in py_iter.enumerate() {
                            let element = iter_result?;
                            if let Some((next_include, next_exclude)) =
                                self.include_or_exclude(py, index, include, exclude)?
                            {
                                items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                            }
                        }
                        Ok(items.into_py(py))
                    }
                    _ => {
                        let iter = SerializationIterator {
                            iterator: py_iter.into_py(py),
                            index: 0,
                            item_serializer: self.item_serializer.as_ref().clone(),
                            extra_owned: ExtraOwned::new(extra),
                            inc_ex: self.inc_ex.clone(),
                            include_arg: include.map(|v| v.into_py(py)),
                            exclude_arg: exclude.map(|v| v.into_py(py)),
                        };
                        Ok(iter.into_py(py))
                    }
                }
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_to_python(value, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.iter() {
            Ok(py_iter) => {
                let len = match value.len() {
                    Ok(len) => Some(len),
                    Err(_) => None,
                };
                let mut seq = serializer.serialize_seq(len)?;
                let item_serializer = self.item_serializer.as_ref();

                for (index, iter_result) in py_iter.enumerate() {
                    let element = iter_result.map_err(py_err_se_err)?;
                    if let Some((next_include, next_exclude)) = self
                        .include_or_exclude(element.py(), index, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        let item_serialize =
                            PydanticSerializer::new(element, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra)
            }
        }
    }
}

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
struct SerializationIterator {
    iterator: Py<PyIterator>,
    #[pyo3(get)]
    index: usize,
    item_serializer: CombinedSerializer,
    extra_owned: ExtraOwned,
    inc_ex: SchemaIncEx<usize>,
    include_arg: Option<PyObject>,
    exclude_arg: Option<PyObject>,
}

#[pymethods]
impl SerializationIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        let iterator = self.iterator.as_ref(py);
        let include = self.include_arg.as_ref().map(|o| o.as_ref(py));
        let exclude = self.exclude_arg.as_ref().map(|o| o.as_ref(py));
        let extra = self.extra_owned.to_extra(py);

        for iter_result in iterator {
            let element = iter_result?;
            let inc_ex =
                self.inc_ex
                    .include_or_exclude(self.index.to_object(py).as_ref(py), self.index, include, exclude)?;
            self.index += 1;
            if let Some((next_include, next_exclude)) = inc_ex {
                let v = self
                    .item_serializer
                    .to_python(element, next_include, next_exclude, &extra)?;
                extra.warnings.final_check(py)?;
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        let iterator = self.iterator.as_ref(py);
        Ok(format!(
            "SerializationIterator(index={}, iterator={})",
            self.index,
            iterator.repr()?
        ))
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        self.__repr__(py)
    }
}
