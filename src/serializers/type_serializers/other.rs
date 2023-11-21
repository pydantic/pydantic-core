use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::py_schema_err;
use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

use super::any::AnySerializer;
use super::{BuildSerializer, CombinedSerializer};

pub struct ChainBuilder;

impl BuildSerializer for ChainBuilder {
    const EXPECTED_TYPE: &'static str = "chain";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let last_schema = schema
            .get_as_req::<Py2<'_, PyList>>(intern2!(schema.py(), "steps"))?
            .iter()
            .last()
            .unwrap()
            .downcast_into()?;
        CombinedSerializer::build(&last_schema, config, definitions)
    }
}

pub struct CustomErrorBuilder;

impl BuildSerializer for CustomErrorBuilder {
    const EXPECTED_TYPE: &'static str = "custom-error";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let sub_schema = schema.get_as_req(intern2!(schema.py(), "schema"))?;
        CombinedSerializer::build(&sub_schema, config, definitions)
    }
}

pub struct CallBuilder;

impl BuildSerializer for CallBuilder {
    const EXPECTED_TYPE: &'static str = "call";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let return_schema = schema.get_as(intern2!(schema.py(), "return_schema"))?;
        match return_schema {
            Some(return_schema) => CombinedSerializer::build(&return_schema, config, definitions),
            None => AnySerializer::build(schema, config, definitions),
        }
    }
}

pub struct LaxOrStrictBuilder;

impl BuildSerializer for LaxOrStrictBuilder {
    const EXPECTED_TYPE: &'static str = "lax-or-strict";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let strict_schema = schema.get_as_req(intern2!(schema.py(), "strict_schema"))?;
        CombinedSerializer::build(&strict_schema, config, definitions)
    }
}

pub struct ArgumentsBuilder;

impl BuildSerializer for ArgumentsBuilder {
    const EXPECTED_TYPE: &'static str = "arguments";

    fn build(
        _schema: &Py2<'_, PyDict>,
        _config: Option<&Py2<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        py_schema_err!("`arguments` validators require a custom serializer")
    }
}

macro_rules! any_build_serializer {
    ($struct_name:ident, $expected_type:literal) => {
        pub struct $struct_name;

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(
                schema: &Py2<'_, PyDict>,
                config: Option<&Py2<'_, PyDict>>,
                definitions: &mut DefinitionsBuilder<CombinedSerializer>,
            ) -> PyResult<CombinedSerializer> {
                AnySerializer::build(schema, config, definitions)
            }
        }
    };
}
any_build_serializer!(IsInstanceBuilder, "is-instance");
any_build_serializer!(IsSubclassBuilder, "is-subclass");
any_build_serializer!(CallableBuilder, "callable");
