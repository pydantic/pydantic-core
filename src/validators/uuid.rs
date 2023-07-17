use pyo3::intern;
use pyo3::prelude::*;
use pyo3::sync::GILOnceCell;
use pyo3::types::{PyDict, PyType};
use uuid::Builder;
use uuid::Uuid;

use crate::build_tools::is_strict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;
use crate::tools::SchemaDict;

use super::model::create_class;
use super::model::force_setattr;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

const UUID_INT: &str = "int";
const UUID_IS_SAFE: &str = "is_safe";

static UUID_TYPE: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn import_type(py: Python, module: &str, attr: &str) -> PyResult<Py<PyType>> {
    py.import(module)?.getattr(attr)?.extract()
}

fn get_uuid_type(py: Python) -> PyResult<&PyType> {
    Ok(UUID_TYPE
        .get_or_init(py, || import_type(py, "uuid", "UUID").unwrap())
        .as_ref(py))
}

#[derive(Debug, Clone, Copy)]
enum Version {
    UUIDv1,
    UUIDv3,
    UUIDv4,
    UUIDv5,
}

impl From<Version> for usize {
    fn from(v: Version) -> Self {
        match v {
            Version::UUIDv1 => 1,
            Version::UUIDv3 => 3,
            Version::UUIDv4 => 4,
            Version::UUIDv5 => 5,
        }
    }
}

impl From<u8> for Version {
    fn from(u: u8) -> Self {
        match u {
            1 => Version::UUIDv1,
            3 => Version::UUIDv3,
            4 => Version::UUIDv4,
            5 => Version::UUIDv5,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UuidValidator {
    strict: bool,
    version: Option<Version>,
}

impl BuildValidator for UuidValidator {
    const EXPECTED_TYPE: &'static str = "uuid";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let version = schema.get_as::<u8>(intern!(py, "version"))?.map(Version::from);
        Ok(Self {
            strict: is_strict(schema, config)?,
            version,
        }
        .into())
    }
}

impl Validator for UuidValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let class = get_uuid_type(py)?;
        if let Some(py_input) = input.input_is_instance(class) {
            Ok(py_input.to_object(py))
        } else if extra.strict.unwrap_or(self.strict) && input.is_python() {
            Err(ValError::new(
                ErrorType::UuidExactType {
                    class_name: self.get_name().to_string(),
                },
                input,
            ))
        } else {
            let dc = create_class(class)?;
            let uuid = self.get_uuid(py, input)?;
            self.set_dict_call(py, dc.as_ref(py), &uuid)?;
            Ok(dc)
        }
    }

    fn different_strict_behavior(
        &self,
        _definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        !ultra_strict
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn complete(&mut self, _definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }
}

impl UuidValidator {
    fn get_uuid<'s, 'data>(&'s self, py: Python<'data>, input: &'data impl Input<'data>) -> ValResult<'data, Uuid> {
        let uuid = match input.validate_str(false) {
            Ok(either_string) => {
                let cow = either_string.as_cow()?;
                let uuid_str = cow.as_ref();
                match Uuid::parse_str(uuid_str) {
                    Ok(uuid) => uuid,
                    Err(e) => return Err(ValError::new(ErrorType::UuidParsing { error: e.to_string() }, input)),
                }
            }
            Err(_) => match input.exact_int() {
                Ok(either_int) => {
                    let int = either_int.into_u128(py)?;
                    Builder::from_u128(int).into_uuid()
                }
                Err(_) => return Err(ValError::new(ErrorType::UuidType, input)),
            },
        };
        if let Some(expected_version) = self.version {
            let v1 = uuid.get_version_num();
            let expected_version = usize::from(expected_version);
            if v1 != expected_version {
                return Err(ValError::new(
                    ErrorType::UuidVersionMismatch {
                        version: v1,
                        schema_version: expected_version,
                    },
                    input,
                ));
            }
        }

        Ok(uuid)
    }

    /// Sets the attributes in a Python dictionary object (`dc`) to represent a UUID.
    /// The function converts the UUID to a u128 integer and sets the corresponding attributes
    /// in the dictionary object to the converted value and a 'safe' flag.
    ///
    /// This implementation does not use the Python `__init__` function to speed up the process,
    /// as the `__init__` function in the Python `uuid` module performs extensive checks.
    fn set_dict_call<'data>(&self, py: Python<'data>, dc: &PyAny, uuid: &Uuid) -> ValResult<'data, ()> {
        let int = uuid.as_u128();
        let safe = py
            .import(intern!(py, "uuid"))?
            .getattr(intern!(py, "SafeUUID"))?
            .get_item("safe")?;
        force_setattr(py, dc, intern!(py, UUID_INT), int)?;
        force_setattr(py, dc, intern!(py, UUID_IS_SAFE), safe)?;
        Ok(())
    }
}
