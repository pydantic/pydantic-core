use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::sync::GILOnceCell;
use pyo3::types::{PyBool, PyDict, PyString};
use pyo3::PyTraverseError;
use pyo3::PyVisit;

use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};
use crate::build_tools::{is_strict, py_schema_err, schema_or_config_same};
use crate::errors::{LocItem, ValError, ValResult};
use crate::input::Input;
use crate::py_gc::PyGcTraverse;
use crate::tools::SchemaDict;
use crate::validators::{Extra, InputType, RecursionState};
use crate::PydanticUndefinedType;
use crate::SchemaError;

static COPY_DEEPCOPY: GILOnceCell<PyObject> = GILOnceCell::new();

fn get_deepcopy(py: Python) -> PyResult<PyObject> {
    Ok(py.import("copy")?.getattr("deepcopy")?.unbind())
}

#[derive(Debug, Clone)]
pub enum DefaultType {
    None,
    Default(PyObject),
    DefaultFactory(PyObject, bool),
}

impl DefaultType {
    pub fn new(schema: &Bound<'_, PyDict>) -> PyResult<Self> {
        let py = schema.py();
        match (
            schema.get_as(intern!(py, "default"))?,
            schema.get_as(intern!(py, "default_factory"))?,
        ) {
            (Some(_), Some(_)) => py_schema_err!("'default' and 'default_factory' cannot be used together"),
            (Some(default), None) => Ok(Self::Default(default)),
            (None, Some(default_factory)) => Ok(Self::DefaultFactory(
                default_factory,
                schema
                    .get_as::<bool>(intern!(py, "default_factory_takes_data"))?
                    .unwrap_or(false),
            )),
            (None, None) => Ok(Self::None),
        }
    }

    pub fn default_value(&self, py: Python, validated_data: Option<&Bound<PyDict>>) -> PyResult<Option<PyObject>> {
        match self {
            Self::Default(ref default) => Ok(Some(default.clone_ref(py))),
            Self::DefaultFactory(ref default_factory, ref takes_data) => {
                let result = if *takes_data {
                    if let Some(data) = validated_data {
                        default_factory.call1(py, (data,))
                    } else {
                        default_factory.call1(py, ({},))
                    }
                } else {
                    default_factory.call0(py)
                };

                Ok(Some(result?))
            }
            Self::None => Ok(None),
        }
    }
}

impl PyGcTraverse for DefaultType {
    fn py_gc_traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Self::Default(obj) | Self::DefaultFactory(obj, _) = self {
            visit.call(obj)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum OnError {
    Raise,
    Omit,
    Default,
}

bitflags::bitflags! {
    #[derive(Debug, Clone)]
    struct ValidateDefaultFlag: u8 {
        const NEVER = 0;
        const INIT = 0x01;
        const DEFINITION = 0x02;
    }
}

impl<'py> FromPyObject<'py> for ValidateDefaultFlag {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(bool_value) = ob.downcast::<PyBool>() {
            Ok(bool_value.is_true().into())
        } else if let Ok(str_value) = ob.extract::<&str>() {
            match str_value {
                "never" => Ok(Self::NEVER),
                "init" => Ok(Self::INIT),
                "definition" => Ok(Self::DEFINITION),
                _ => Err(PyValueError::new_err(
                    "Invalid value for option `validate_default`, should be `'init'`, `'definition'`, `'never'` or a `bool`",
                )),
            }
        } else {
            Err(PyTypeError::new_err(
                "Invalid value for option `validate_default`, should be `'init'`, `'definition'`, `'never'` or a `bool`",
            ))
        }
    }
}

impl From<bool> for ValidateDefaultFlag {
    fn from(mode: bool) -> Self {
        if mode {
            Self::INIT
        } else {
            Self::NEVER
        }
    }
}

impl Default for ValidateDefaultFlag {
    fn default() -> Self {
        Self::NEVER
    }
}

#[derive(Debug)]
pub struct WithDefaultValidator {
    default: DefaultType,
    on_error: OnError,
    validator: Box<CombinedValidator>,
    validate_default: ValidateDefaultFlag,
    copy_default: bool,
    name: String,
    undefined: PyObject,
}

impl BuildValidator for WithDefaultValidator {
    const EXPECTED_TYPE: &'static str = "default";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let default = DefaultType::new(schema)?;
        let on_error = match schema
            .get_as::<Bound<'_, PyString>>(intern!(py, "on_error"))?
            .as_ref()
            .map(|s| s.to_str())
            .transpose()?
        {
            Some("raise") => OnError::Raise,
            Some("omit") => OnError::Omit,
            Some("default") => {
                if matches!(default, DefaultType::None) {
                    return py_schema_err!("'on_error = default' requires a `default` or `default_factory`");
                }
                OnError::Default
            }
            None => OnError::Raise,
            // schema validation means other values are impossible
            _ => unreachable!(),
        };

        let sub_schema = schema.get_as_req(intern!(schema.py(), "schema"))?;
        let validator = Box::new(build_validator(&sub_schema, config, definitions)?);

        let copy_default = if let DefaultType::Default(default_obj) = &default {
            default_obj.bind(py).hash().is_err()
        } else {
            false
        };

        let name = format!("{}[{}]", Self::EXPECTED_TYPE, validator.get_name());
        let validate_default =
            schema_or_config_same(schema, config, intern!(py, "validate_default"))?.unwrap_or_default();
        let validator = Self {
            default,
            on_error,
            validator,
            validate_default,
            copy_default,
            name,
            undefined: PydanticUndefinedType::new(py).to_object(py),
        };

        validator.validate_default_on_build(schema, config)?;

        Ok(validator.into())
    }
}

impl_py_gc_traverse!(WithDefaultValidator { default, validator });

impl Validator for WithDefaultValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        if input.as_python().is_some_and(|py_input| py_input.is(&self.undefined)) {
            Ok(self.default_value(py, None::<usize>, state)?.unwrap())
        } else {
            match self.validator.validate(py, input, state) {
                Ok(v) => Ok(v),
                Err(e) => match e {
                    ValError::UseDefault => Ok(self.default_value(py, None::<usize>, state)?.ok_or(e)?),
                    e => match self.on_error {
                        OnError::Raise => Err(e),
                        OnError::Default => Ok(self.default_value(py, None::<usize>, state)?.ok_or(e)?),
                        OnError::Omit => Err(ValError::Omit),
                    },
                },
            }
        }
    }

    fn default_value<'py>(
        &self,
        py: Python<'py>,
        outer_loc: Option<impl Into<LocItem>>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<Option<PyObject>> {
        match self.default.default_value(py, state.extra().data.as_ref())? {
            Some(stored_dft) => {
                let dft: Py<PyAny> = if self.copy_default {
                    let deepcopy_func = COPY_DEEPCOPY.get_or_init(py, || get_deepcopy(py).unwrap());
                    deepcopy_func.call1(py, (&stored_dft,))?
                } else {
                    stored_dft
                };
                if self.validate_default.contains(ValidateDefaultFlag::INIT) {
                    self.validate_default(py, outer_loc, state, dft)
                } else {
                    Ok(Some(dft))
                }
            }
            None => Ok(None),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl WithDefaultValidator {
    pub fn has_default(&self) -> bool {
        !matches!(self.default, DefaultType::None)
    }

    pub fn omit_on_error(&self) -> bool {
        matches!(self.on_error, OnError::Omit)
    }

    fn validate_default<'py>(
        &self,
        py: Python<'py>,
        outer_loc: Option<impl Into<LocItem>>,
        state: &mut ValidationState<'_, 'py>,
        dft: Py<PyAny>,
    ) -> ValResult<Option<PyObject>> {
        match self.validate(py, dft.bind(py), state) {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                if let Some(outer_loc) = outer_loc {
                    Err(e.with_outer_location(outer_loc))
                } else {
                    Err(e)
                }
            }
        }
    }

    fn validate_default_on_build(
        &self,
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if self.validate_default.contains(ValidateDefaultFlag::DEFINITION) && self.has_default() {
            // Since this method is called in `build` where validation state is not available,
            // we need to craft a dummy one here. This setup is basically the same as in `SchemaValidator::get_default_value`
            let mut recursion_guard = RecursionState::default();
            let mut state = ValidationState::new(
                Extra::new(
                    Some(is_strict(schema, config)?),
                    None,
                    None,
                    None,
                    InputType::Python,
                    true.into(),
                ),
                &mut recursion_guard,
                false.into(),
            );
            let py = schema.py();
            if let Some(dft) = self.default.default_value(py, state.extra().data.as_ref())? {
                if let Err(e) = self.validate_default(py, None::<usize>, &mut state, dft) {
                    return Err(SchemaError::from_val_error(py, e));
                }
            }
        }
        Ok(())
    }
}
