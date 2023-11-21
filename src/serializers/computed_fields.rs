use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use pyo3::{intern2, PyTraverseError, PyVisit};
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::build_tools::py_schema_error_type;
use crate::definitions::DefinitionsBuilder;
use crate::py_gc::PyGcTraverse;
use crate::serializers::filter::SchemaFilter;
use crate::serializers::shared::{BuildSerializer, CombinedSerializer, PydanticSerializer, TypeSerializer};
use crate::tools::SchemaDict;

use super::errors::py_err_se_err;
use super::Extra;

#[derive(Debug, Clone)]
pub(super) struct ComputedFields(Vec<ComputedField>);

impl ComputedFields {
    pub fn new(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<Option<Self>> {
        let py = schema.py();
        if let Some(computed_fields) = schema.get_as::<Py2<'_, PyList>>(intern2!(py, "computed_fields"))? {
            let computed_fields = computed_fields
                .iter()
                .map(|field| ComputedField::new(&field, config, definitions))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Some(Self(computed_fields)))
        } else {
            Ok(None)
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_python(
        &self,
        model: &Py2<'_, PyAny>,
        output_dict: &Py2<'_, PyDict>,
        filter: &SchemaFilter<isize>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<()> {
        if extra.round_trip {
            // Do not serialize computed fields
            return Ok(());
        }
        for computed_fields in &self.0 {
            computed_fields.to_python(model, output_dict, filter, include, exclude, extra)?;
        }
        Ok(())
    }

    pub fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        model: &Py2<'_, PyAny>,
        map: &mut S::SerializeMap,
        filter: &SchemaFilter<isize>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<(), S::Error> {
        if extra.round_trip {
            // Do not serialize computed fields
            return Ok(());
        }
        for computed_field in &self.0 {
            let property_name_py = computed_field.property_name_py.attach(model.py());
            let value = model.getattr(property_name_py).map_err(py_err_se_err)?;
            if extra.exclude_none && PyAnyMethods::is_none(&value) {
                return Ok(());
            }
            if let Some((next_include, next_exclude)) = filter
                .key_filter(property_name_py, include, exclude)
                .map_err(py_err_se_err)?
            {
                let cfs = ComputedFieldSerializer {
                    model,
                    computed_field,
                    include: next_include.as_ref(),
                    exclude: next_exclude.as_ref(),
                    extra,
                };
                let key = match extra.by_alias {
                    true => computed_field.alias.as_str(),
                    false => computed_field.property_name.as_str(),
                };
                map.serialize_entry(key, &cfs)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ComputedField {
    property_name: String,
    property_name_py: Py<PyString>,
    serializer: CombinedSerializer,
    alias: String,
    alias_py: Py<PyString>,
}

impl ComputedField {
    pub fn new(
        schema: &Py2<'_, PyAny>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<Self> {
        let py = schema.py();
        let schema: &Py2<'_, PyDict> = schema.downcast()?;
        let property_name: Py2<'_, PyString> = schema.get_as_req(intern2!(py, "property_name"))?;
        let return_schema = schema.get_as_req(intern2!(py, "return_schema"))?;
        let serializer = CombinedSerializer::build(&return_schema, config, definitions)
            .map_err(|e| py_schema_error_type!("Computed field `{}`:\n  {}", property_name, e))?;
        let alias_py = schema
            .get_as(intern2!(py, "alias"))?
            .unwrap_or_else(|| property_name.clone());
        Ok(Self {
            property_name: property_name.extract()?,
            property_name_py: property_name.into_py(py),
            serializer,
            alias: alias_py.extract()?,
            alias_py: alias_py.into_py(py),
        })
    }

    fn to_python(
        &self,
        model: &Py2<'_, PyAny>,
        output_dict: &Py2<'_, PyDict>,
        filter: &SchemaFilter<isize>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<()> {
        let py = model.py();
        let property_name_py = self.property_name_py.attach(py);

        if let Some((next_include, next_exclude)) = filter.key_filter(property_name_py, include, exclude)? {
            let next_value = model.getattr(property_name_py)?;

            let value = self
                .serializer
                .to_python(&next_value, next_include.as_ref(), next_exclude.as_ref(), extra)?;
            if extra.exclude_none && value.is_none(py) {
                return Ok(());
            }
            let key = match extra.by_alias {
                true => self.alias_py.attach(py),
                false => property_name_py,
            };
            output_dict.set_item(key, value)?;
        }
        Ok(())
    }
}

pub(crate) struct ComputedFieldSerializer<'py> {
    model: &'py Py2<'py, PyAny>,
    computed_field: &'py ComputedField,
    include: Option<&'py Py2<'py, PyAny>>,
    exclude: Option<&'py Py2<'py, PyAny>>,
    extra: &'py Extra<'py>,
}

impl_py_gc_traverse!(ComputedField { serializer });

impl PyGcTraverse for ComputedFields {
    fn py_gc_traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.0.py_gc_traverse(visit)
    }
}

impl_py_gc_traverse!(ComputedFieldSerializer<'_> { computed_field });

impl<'py> Serialize for ComputedFieldSerializer<'py> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let py = self.model.py();
        let property_name_py = self.computed_field.property_name_py.as_ref(py);
        let next_value = self.model.getattr(property_name_py).map_err(py_err_se_err)?;
        let s = PydanticSerializer::new(
            &next_value,
            &self.computed_field.serializer,
            self.include,
            self.exclude,
            self.extra,
        );
        s.serialize(serializer)
    }
}
