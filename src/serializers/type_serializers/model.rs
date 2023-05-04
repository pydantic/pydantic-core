use std::borrow::Cow;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyType};

use ahash::AHashMap;

use crate::build_tools::{py_error_type, ExtraBehavior, SchemaDict};
use crate::definitions::DefinitionsBuilder;

use super::{
    infer_json_key, infer_json_key_known, infer_serialize, infer_to_python, object_to_dict, py_err_se_err,
    BuildSerializer, CombinedSerializer, ComputedFields, Extra, GeneralFieldsSerializer, ObType, SerCheck, SerField,
    TypeSerializer,
};

pub struct ModelFieldsBuilder;

impl BuildSerializer for ModelFieldsBuilder {
    const EXPECTED_TYPE: &'static str = "model-fields";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let include_extra = matches!(
            ExtraBehavior::from_schema_or_config(py, schema, config, ExtraBehavior::Ignore)?,
            ExtraBehavior::Allow
        );

        let fields_dict: &PyDict = schema.get_as_req(intern!(py, "fields"))?;
        let mut fields: AHashMap<String, SerField> = AHashMap::with_capacity(fields_dict.len());

        for (key, value) in fields_dict.iter() {
            let key_py: &PyString = key.downcast()?;
            let key: String = key_py.extract()?;
            let field_info: &PyDict = value.downcast()?;

            let key_py: Py<PyString> = key_py.into_py(py);

            if field_info.get_as(intern!(py, "serialization_exclude"))? == Some(true) {
                fields.insert(key, SerField::new(py, key_py, None, None, true));
            } else {
                let alias: Option<String> = field_info.get_as(intern!(py, "serialization_alias"))?;

                let schema = field_info.get_as_req(intern!(py, "schema"))?;
                let serializer = CombinedSerializer::build(schema, config, definitions)
                    .map_err(|e| py_error_type!("Field `{}`:\n  {}", key, e))?;

                fields.insert(key, SerField::new(py, key_py, alias, Some(serializer), true));
            }
        }

        let computed_fields = ComputedFields::new(schema)?;

        Ok(GeneralFieldsSerializer::new(fields, include_extra, computed_fields).into())
    }
}

#[derive(Debug, Clone)]
pub struct ModelSerializer {
    class: Py<PyType>,
    serializer: Box<CombinedSerializer>,
    name: String,
}

impl BuildSerializer for ModelSerializer {
    const EXPECTED_TYPE: &'static str = "model";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let class: &PyType = schema.get_as_req(intern!(py, "cls"))?;
        let sub_schema: &PyDict = schema.get_as_req(intern!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(sub_schema, config, definitions)?);

        Ok(Self {
            class: class.into(),
            serializer,
            name: class.getattr(intern!(py, "__name__"))?.extract()?,
        }
        .into())
    }
}

impl ModelSerializer {
    fn allow_value(&self, value: &PyAny, extra: &Extra) -> PyResult<bool> {
        match extra.check {
            SerCheck::Strict => Ok(value.get_type().is(self.class.as_ref(value.py()))),
            SerCheck::Lax => value.is_instance(self.class.as_ref(value.py())),
            SerCheck::None => value.hasattr(intern!(value.py(), "__dict__")),
        }
    }
}

impl TypeSerializer for ModelSerializer {
    fn py_gc_traverse(&self, visit: &pyo3::PyVisit<'_>) -> Result<(), pyo3::PyTraverseError> {
        visit.call(&self.class)?;
        self.serializer.py_gc_traverse(visit)?;
        Ok(())
    }

    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let extra = Extra {
            model: Some(value),
            ..*extra
        };
        if self.allow_value(value, &extra)? {
            let dict = object_to_dict(value, true, &extra)?;
            if let Some(pydantic_extra) = get_pydantic_extra(value) {
                let py_tuple = (dict, pydantic_extra).to_object(py);
                self.serializer.to_python(py_tuple.as_ref(py), include, exclude, &extra)
            } else {
                self.serializer.to_python(dict, include, exclude, &extra)
            }
        } else {
            extra.warnings.on_fallback_py(self.get_name(), value, &extra)?;
            infer_to_python(value, include, exclude, &extra)
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        if self.allow_value(key, extra)? {
            infer_json_key_known(&ObType::PydanticSerializable, key, extra)
        } else {
            extra.warnings.on_fallback_py(&self.name, key, extra)?;
            infer_json_key(key, extra)
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
        let extra = Extra {
            model: Some(value),
            ..*extra
        };
        if self.allow_value(value, &extra).map_err(py_err_se_err)? {
            let dict = object_to_dict(value, true, &extra).map_err(py_err_se_err)?;
            self.serializer
                .serde_serialize(dict, serializer, include, exclude, &extra)
        } else {
            extra.warnings.on_fallback_ser::<S>(self.get_name(), value, &extra)?;
            infer_serialize(value, serializer, include, exclude, &extra)
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn retry_with_lax_check(&self) -> bool {
        true
    }
}

fn get_pydantic_extra(value: &PyAny) -> Option<&PyDict> {
    let extra = match value.getattr(intern!(value.py(), "__pydantic_extra__")) {
        Ok(extra) => extra,
        Err(_) => return None,
    };
    match extra.downcast::<PyDict>() {
        Ok(attrs) => Some(attrs),
        Err(_) => None,
    }
}
