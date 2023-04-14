use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

use serde::ser::SerializeMap;
use serde::Serialize;

use crate::build_tools::SchemaDict;

use super::errors::py_err_se_err;
use super::infer::{infer_serialize, infer_serialize_known, infer_to_python, infer_to_python_known};
use super::ob_type::ObType;
use super::{Extra, SerMode};

use super::type_serializers::function::get_json_return_type;

#[derive(Debug, Clone)]
pub(super) struct ComputedFields(Vec<ComputedField>);

impl ComputedFields {
    pub fn new(schema: &PyDict) -> PyResult<Option<Self>> {
        let py = schema.py();
        if let Some(computed_fields) = schema.get_as::<&PyList>(intern!(py, "computed_fields"))? {
            let computed_fields = computed_fields
                .iter()
                .map(ComputedField::new)
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Some(Self(computed_fields)))
        } else {
            Ok(None)
        }
    }

    pub fn to_python(
        &self,
        model: &PyAny,
        output_dict: &PyDict,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<()> {
        for computed_fields in self.0.iter() {
            computed_fields.to_python(model, output_dict, include, exclude, extra)?;
        }
        Ok(())
    }

    pub fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        model: &PyAny,
        map: &mut S::SerializeMap,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<(), S::Error> {
        for computed_field in self.0.iter() {
            let cfs = ComputedFieldSerializer {
                model,
                computed_field,
                include,
                exclude,
                extra,
            };
            map.serialize_entry(&computed_field.alias, &cfs)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ComputedField {
    property_name: Py<PyString>,
    return_ob_type: Option<ObType>,
    alias: String,
    alias_py: Py<PyString>,
}

impl ComputedField {
    pub fn new(schema: &PyAny) -> PyResult<Self> {
        let py = schema.py();
        let schema: &PyDict = schema.downcast()?;
        let property_name: &PyString = schema.get_as_req(intern!(py, "property_name"))?;
        let return_ob_type = get_json_return_type(schema)?;
        let alias_py: &PyString = schema.get_as(intern!(py, "alias"))?.unwrap_or(property_name);
        Ok(Self {
            property_name: property_name.into_py(py),
            return_ob_type,
            alias: alias_py.extract()?,
            alias_py: alias_py.into_py(py),
        })
    }

    fn to_python(
        &self,
        model: &PyAny,
        output_dict: &PyDict,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<()> {
        let py = model.py();
        let property_name = self.property_name.as_ref(py);
        let next_value = model.getattr(property_name)?;

        // TODO fix include & exclude
        let value = match extra.mode {
            SerMode::Json => match self.return_ob_type {
                Some(ref ob_type) => infer_to_python_known(ob_type, next_value, include, exclude, extra),
                None => infer_to_python(next_value, include, exclude, extra),
            },
            _ => Ok(next_value.to_object(py)),
        }?;
        output_dict.set_item(self.alias_py.as_ref(py), value)?;
        Ok(())
    }
}

pub(crate) struct ComputedFieldSerializer<'py> {
    model: &'py PyAny,
    computed_field: &'py ComputedField,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
    extra: &'py Extra<'py>,
}

impl<'py> Serialize for ComputedFieldSerializer<'py> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let py = self.model.py();
        let property_name = self.computed_field.property_name.as_ref(py);
        let next_value = self.model.getattr(property_name).map_err(py_err_se_err)?;

        match self.computed_field.return_ob_type {
            Some(ref ob_type) => {
                infer_serialize_known(ob_type, next_value, serializer, self.include, self.exclude, self.extra)
            }
            None => infer_serialize(next_value, serializer, self.include, self.exclude, self.extra),
        }
    }
}
