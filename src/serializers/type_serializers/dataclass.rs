use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyType};
use std::borrow::Cow;

use ahash::AHashMap;

use crate::build_tools::{py_schema_error_type, ExtraBehavior};
use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

use super::{
    infer_json_key, infer_json_key_known, infer_serialize, infer_to_python, py_err_se_err, BuildSerializer,
    CombinedSerializer, ComputedFields, Extra, FieldsMode, GeneralFieldsSerializer, ObType, SerCheck, SerField,
    TypeSerializer,
};

pub struct DataclassArgsBuilder;

impl BuildSerializer for DataclassArgsBuilder {
    const EXPECTED_TYPE: &'static str = "dataclass-args";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let fields_list: Py2<'_, PyList> = schema.get_as_req(intern2!(py, "fields"))?;
        let mut fields: AHashMap<String, SerField> = AHashMap::with_capacity(fields_list.len());

        let fields_mode = match ExtraBehavior::from_schema_or_config(py, schema, config, ExtraBehavior::Ignore)? {
            ExtraBehavior::Allow => FieldsMode::TypedDictAllow,
            _ => FieldsMode::SimpleDict,
        };

        for (index, item) in fields_list.iter().enumerate() {
            let field_info = item.downcast::<PyDict>()?;
            let name: String = field_info.get_as_req(intern2!(py, "name"))?;

            let key_py: Py<PyString> = PyString::new2(py, &name).into();

            if field_info.get_as(intern2!(py, "serialization_exclude"))? == Some(true) {
                fields.insert(name, SerField::new(py, key_py, None, None, true));
            } else {
                let schema = field_info.get_as_req(intern2!(py, "schema"))?;
                let serializer = CombinedSerializer::build(&schema, config, definitions)
                    .map_err(|e| py_schema_error_type!("Field `{}`:\n  {}", index, e))?;

                let alias = field_info.get_as(intern2!(py, "serialization_alias"))?;
                fields.insert(name, SerField::new(py, key_py, alias, Some(serializer), true));
            }
        }

        let computed_fields = ComputedFields::new(schema, config, definitions)?;

        Ok(GeneralFieldsSerializer::new(fields, fields_mode, None, computed_fields).into())
    }
}

#[derive(Debug, Clone)]
pub struct DataclassSerializer {
    class: Py<PyType>,
    serializer: Box<CombinedSerializer>,
    fields: Vec<Py<PyString>>,
    name: String,
}

impl BuildSerializer for DataclassSerializer {
    const EXPECTED_TYPE: &'static str = "dataclass";

    fn build(
        schema: &Py2<'_, PyDict>,
        _config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        // models ignore the parent config and always use the config from this model
        let config = schema.get_as(intern2!(py, "config"))?;

        let class: &PyType = schema.get_as_req(intern2!(py, "cls"))?;
        let sub_schema = schema.get_as_req(intern2!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(&sub_schema, config.as_ref(), definitions)?);

        let fields = schema
            .get_as_req::<&PyList>(intern2!(py, "fields"))?
            .iter()
            .map(|s| Ok(s.downcast::<PyString>()?.into_py(py)))
            .collect::<PyResult<Vec<_>>>()?;

        Ok(Self {
            class: class.into(),
            serializer,
            fields,
            name: class.getattr(intern2!(py, "__name__"))?.extract()?,
        }
        .into())
    }
}

impl DataclassSerializer {
    fn allow_value(&self, value: &Py2<'_, PyAny>, extra: &Extra) -> PyResult<bool> {
        match extra.check {
            SerCheck::Strict => Ok(value.get_type().is(self.class.attach(value.py()))),
            SerCheck::Lax => value.is_instance(self.class.attach(value.py())),
            SerCheck::None => value.hasattr(intern2!(value.py(), "__dataclass_fields__")),
        }
    }

    fn get_inner_value<'py>(&self, value: &Py2<'py, PyAny>) -> PyResult<Py2<'py, PyDict>> {
        let py = value.py();
        let dict = PyDict::new2(py);

        for field_name in &self.fields {
            let field_name = field_name.as_ref(py);
            dict.set_item(field_name, value.getattr(field_name)?)?;
        }
        Ok(dict)
    }
}

impl_py_gc_traverse!(DataclassSerializer { class, serializer });

impl TypeSerializer for DataclassSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let extra = Extra {
            model: Some(value),
            ..*extra
        };
        if self.allow_value(value, &extra)? {
            let inner_value = self.get_inner_value(value)?;
            self.serializer.to_python(&inner_value, include, exclude, &extra)
        } else {
            extra.warnings.on_fallback_py(self.get_name(), value, &extra)?;
            infer_to_python(value, include, exclude, &extra)
        }
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        if self.allow_value(key, extra)? {
            infer_json_key_known(ObType::Dataclass, key, extra)
        } else {
            extra.warnings.on_fallback_py(&self.name, key, extra)?;
            infer_json_key(key, extra)
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let extra = Extra {
            model: Some(value),
            ..*extra
        };
        if self.allow_value(value, &extra).map_err(py_err_se_err)? {
            let inner_value = self.get_inner_value(value).map_err(py_err_se_err)?;
            self.serializer
                .serde_serialize(&inner_value, serializer, include, exclude, &extra)
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
