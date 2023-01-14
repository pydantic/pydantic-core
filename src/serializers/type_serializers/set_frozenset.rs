use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PyList, PySet};

use serde::ser::SerializeSeq;

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::{BuildSerializer, CombinedSerializer, Extra, PydanticSerializer, SerMode, TypeSerializer};

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal, $py_type:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            item_serializer: Box<CombinedSerializer>,
        }

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

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
                }
                .into())
            }
        }

        impl TypeSerializer for $struct_name {
            fn to_python(
                &self,
                value: &PyAny,
                include: Option<&PyAny>,
                exclude: Option<&PyAny>,
                extra: &Extra,
                error_on_fallback: bool,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match value.cast_as::<$py_type>() {
                    Ok(py_set) => {
                        let item_serializer = self.item_serializer.as_ref();

                        let mut items = Vec::with_capacity(py_set.len());
                        for element in py_set.iter() {
                            items.push(item_serializer.to_python(
                                element,
                                include,
                                exclude,
                                extra,
                                error_on_fallback,
                            )?);
                        }
                        match extra.mode {
                            SerMode::Json => Ok(PyList::new(py, items).into_py(py)),
                            _ => Ok(<$py_type>::new(py, &items)?.into_py(py)),
                        }
                    }
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_py(Self::EXPECTED_TYPE, value, error_on_fallback)?;
                        fallback_to_python(value, include, exclude, extra, error_on_fallback)
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
                error_on_fallback: bool,
            ) -> Result<S::Ok, S::Error> {
                match value.cast_as::<$py_type>() {
                    Ok(py_set) => {
                        let mut seq = serializer.serialize_seq(Some(py_set.len()))?;
                        let item_serializer = self.item_serializer.as_ref();

                        for value in py_set.iter() {
                            let item_serialize = PydanticSerializer::new(
                                value,
                                item_serializer,
                                include,
                                exclude,
                                extra,
                                error_on_fallback,
                            );
                            seq.serialize_element(&item_serialize)?;
                        }
                        seq.end()
                    }
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_ser::<S>(Self::EXPECTED_TYPE, value, error_on_fallback)?;
                        fallback_serialize(value, serializer, include, exclude, extra, error_on_fallback)
                    }
                }
            }
        }
    };
}

build_serializer!(SetSerializer, "set", PySet);
build_serializer!(FrozenSetSerializer, "frozenset", PyFrozenSet);
