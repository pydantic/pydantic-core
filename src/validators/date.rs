use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDate, PyDict, PyTime};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValError, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DateValidator {
    strict: bool,
    le: Option<Py<PyDate>>,
    lt: Option<Py<PyDate>>,
    ge: Option<Py<PyDate>>,
    gt: Option<Py<PyDate>>,
}

impl BuildValidator for DateValidator {
    const EXPECTED_TYPE: &'static str = "date";

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

impl Validator for DateValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let date = match self.strict {
            true => input.strict_date(py)?,
            false => {
                match input.lax_date(py) {
                    Ok(date) => date,
                    Err(date_err) => {
                        let dt = match input.lax_datetime(py) {
                            Ok(dt) => dt,
                            Err(dt_err) => {
                                return match dt_err {
                                    ValError::LineErrors(mut line_errors) => {
                                        for line_error in line_errors.iter_mut() {
                                            match line_error.kind {
                                                ErrorKind::DateTimeParsing => {
                                                    line_error.kind = ErrorKind::DateFromDatetimeParsing;
                                                }
                                                _ => {
                                                    return Err(date_err);
                                                }
                                            }
                                        }
                                        Err(ValError::LineErrors(line_errors))
                                    }
                                    ValError::InternalErr(internal_err) => Err(ValError::InternalErr(internal_err)),
                                };
                            }
                        };
                        // TODO replace all this with raw rust types once github.com/samuelcolvin/speedate#6 is done

                        // we want to make sure the time is zero - e.g. the dt is an "exact date"
                        let dt_time: &PyTime = dt
                            .call_method0("time")
                            .map_err(as_internal)?
                            .extract()
                            .map_err(as_internal)?;

                        let zero_time = PyTime::new(py, 0, 0, 0, 0, None).map_err(as_internal)?;
                        if dt_time.eq(zero_time).map_err(as_internal)? {
                            dt.call_method0("date")
                                .map_err(as_internal)?
                                .extract()
                                .map_err(as_internal)?
                        } else {
                            return err_val_error!(
                                input_value = InputValue::InputRef(input),
                                kind = ErrorKind::DateFromDatetimeInexact
                            );
                        }
                    }
                }
            }
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
        self.validation_comparison(py, input, input.strict_date(py)?)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DateValidator {
    fn validation_comparison<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        date: &'data PyDate,
    ) -> ValResult<'data, PyObject> {
        macro_rules! check_constraint {
            ($constraint_op:expr, $op:path, $error:path, $key:literal) => {
                if let Some(constraint_py) = &$constraint_op {
                    let constraint: &PyDate = constraint_py.extract(py).map_err(as_internal)?;
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
