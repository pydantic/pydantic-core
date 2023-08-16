use pyo3::intern;
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
        schema: &PyDict,
        user_config: &crate::user_config::UserConfig,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let last_schema = schema
            .get_as_req::<&PyList>(intern!(schema.py(), "steps"))?
            .iter()
            .last()
            .unwrap()
            .downcast()?;
        CombinedSerializer::build(last_schema, user_config, definitions)
    }
}

pub struct CustomErrorBuilder;

impl BuildSerializer for CustomErrorBuilder {
    const EXPECTED_TYPE: &'static str = "custom-error";

    fn build(
        schema: &PyDict,
        user_config: &crate::user_config::UserConfig,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let sub_schema: &PyDict = schema.get_as_req(intern!(schema.py(), "schema"))?;
        CombinedSerializer::build(sub_schema, user_config, definitions)
    }
}

pub struct CallBuilder;

impl BuildSerializer for CallBuilder {
    const EXPECTED_TYPE: &'static str = "call";

    fn build(
        schema: &PyDict,
        user_config: &crate::user_config::UserConfig,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let return_schema = schema.get_as::<&PyDict>(intern!(schema.py(), "return_schema"))?;
        match return_schema {
            Some(return_schema) => CombinedSerializer::build(return_schema, user_config, definitions),
            None => AnySerializer::build(schema, user_config, definitions),
        }
    }
}

pub struct LaxOrStrictBuilder;

impl BuildSerializer for LaxOrStrictBuilder {
    const EXPECTED_TYPE: &'static str = "lax-or-strict";

    fn build(
        schema: &PyDict,
        user_config: &crate::user_config::UserConfig,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let strict_schema: &PyDict = schema.get_as_req(intern!(schema.py(), "strict_schema"))?;
        CombinedSerializer::build(strict_schema, user_config, definitions)
    }
}

pub struct ArgumentsBuilder;

impl BuildSerializer for ArgumentsBuilder {
    const EXPECTED_TYPE: &'static str = "arguments";

    fn build(
        _schema: &PyDict,
        _user_config: &crate::user_config::UserConfig,
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
                schema: &PyDict,
                user_config: &crate::user_config::UserConfig,
                definitions: &mut DefinitionsBuilder<CombinedSerializer>,
            ) -> PyResult<CombinedSerializer> {
                AnySerializer::build(schema, user_config, definitions)
            }
        }
    };
}
any_build_serializer!(IsInstanceBuilder, "is-instance");
any_build_serializer!(IsSubclassBuilder, "is-subclass");
any_build_serializer!(CallableBuilder, "callable");
