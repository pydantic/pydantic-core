use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDateTime, PyDict};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DateTimeValidator {
    strict: bool,
    le: Option<Py<PyDateTime>>,
    lt: Option<Py<PyDateTime>>,
    ge: Option<Py<PyDateTime>>,
    gt: Option<Py<PyDateTime>>,
}

impl BuildValidator for DateTimeValidator {
    const EXPECTED_TYPE: &'static str = "datetime";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            le: schema.get_as("le")?,
            lt: schema.get_as("lt")?,
            ge: schema.get_as("ge")?,
            gt: schema.get_as("gt")?,
        }
        .into())
    }
}

impl Validator for DateTimeValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let date = match self.strict {
            true => input.strict_datetime(py)?,
            false => input.lax_datetime(py)?,
        };
        self.validation_comparison(py, input, date)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        self.validation_comparison(py, input, input.strict_datetime(py)?)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DateTimeValidator {
    fn validation_comparison<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        date: &'data PyDateTime,
    ) -> ValResult<'data, PyObject> {
        macro_rules! check_constraint {
            ($constraint_op:expr, $op:path, $error:path, $key:literal) => {
                if let Some(constraint_py) = &$constraint_op {
                    let constraint: &PyDateTime = constraint_py.extract(py).map_err(as_internal)?;
                    let comparison_py = date.rich_compare(constraint, $op).map_err(as_internal)?;
                    let comparison: bool = comparison_py.extract().map_err(as_internal)?;
                    if !comparison {
                        return err_val_error!(
                            input_value = InputValue::InputRef(input),
                            kind = $error,
                            context = context!($key => constraint.to_string())
                        );
                    }
                }
            };
        }

        check_constraint!(self.le, CompareOp::Le, ErrorKind::LessThanEqual, "le");
        check_constraint!(self.lt, CompareOp::Lt, ErrorKind::LessThan, "lt");
        check_constraint!(self.ge, CompareOp::Ge, ErrorKind::GreaterThanEqual, "ge");
        check_constraint!(self.gt, CompareOp::Gt, ErrorKind::GreaterThan, "gt");
        Ok(date.into_py(py))
    }
}
