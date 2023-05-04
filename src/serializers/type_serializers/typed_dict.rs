use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use ahash::AHashMap;

use crate::build_tools::{py_error_type, schema_or_config, ExtraBehavior, SchemaDict};
use crate::definitions::DefinitionsBuilder;

use super::{BuildSerializer, CombinedSerializer, ComputedFields, GeneralFieldsSerializer, SerField};

#[derive(Debug, Clone)]
pub struct TypedDictBuilder;

impl BuildSerializer for TypedDictBuilder {
    const EXPECTED_TYPE: &'static str = "typed-dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let total =
            schema_or_config(schema, config, intern!(py, "total"), intern!(py, "typed_dict_total"))?.unwrap_or(true);

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
            let required = field_info.get_as(intern!(py, "required"))?.unwrap_or(total);

            if field_info.get_as(intern!(py, "serialization_exclude"))? == Some(true) {
                fields.insert(key, SerField::new(py, key_py, None, None, required));
            } else {
                let alias: Option<String> = field_info.get_as(intern!(py, "serialization_alias"))?;

                let schema = field_info.get_as_req(intern!(py, "schema"))?;
                let serializer = CombinedSerializer::build(schema, config, definitions)
                    .map_err(|e| py_error_type!("Field `{}`:\n  {}", key, e))?;
                fields.insert(key, SerField::new(py, key_py, alias, Some(serializer), required));
            }
        }

        let computed_fields = ComputedFields::new(schema)?;

        Ok(GeneralFieldsSerializer::new(fields, include_extra, computed_fields).into())
    }
}
