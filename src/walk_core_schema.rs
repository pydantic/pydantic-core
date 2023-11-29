use pyo3::exceptions::PyTypeError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

#[pyclass(subclass, module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct WalkCoreSchema {
    // core schemas, see CoreSchema in core_schema.py
    visit_any_schema: Option<PyObject>,
    visit_none_schema: Option<PyObject>,
    visit_bool_schema: Option<PyObject>,
    visit_int_schema: Option<PyObject>,
    visit_float_schema: Option<PyObject>,
    visit_decimal_schema: Option<PyObject>,
    visit_string_schema: Option<PyObject>,
    visit_bytes_schema: Option<PyObject>,
    visit_date_schema: Option<PyObject>,
    visit_time_schema: Option<PyObject>,
    visit_datetime_schema: Option<PyObject>,
    visit_timedelta_schema: Option<PyObject>,
    visit_literal_schema: Option<PyObject>,
    visit_is_instance_schema: Option<PyObject>,
    visit_is_subclass_schema: Option<PyObject>,
    visit_callable_schema: Option<PyObject>,
    visit_list_schema: Option<PyObject>,
    visit_tuple_positional_schema: Option<PyObject>,
    visit_tuple_variable_schema: Option<PyObject>,
    visit_set_schema: Option<PyObject>,
    visit_frozenset_schema: Option<PyObject>,
    visit_generator_schema: Option<PyObject>,
    visit_dict_schema: Option<PyObject>,
    visit_after_validator_function_schema: Option<PyObject>,
    visit_before_validator_function_schema: Option<PyObject>,
    visit_wrap_validator_function_schema: Option<PyObject>,
    visit_plain_validator_function_schema: Option<PyObject>,
    visit_with_default_schema: Option<PyObject>,
    visit_nullable_schema: Option<PyObject>,
    visit_union_schema: Option<PyObject>,
    visit_tagged_union_schema: Option<PyObject>,
    visit_chain_schema: Option<PyObject>,
    visit_lax_or_strict_schema: Option<PyObject>,
    visit_json_or_python_schema: Option<PyObject>,
    visit_typed_dict_schema: Option<PyObject>,
    visit_model_fields_schema: Option<PyObject>,
    visit_model_schema: Option<PyObject>,
    visit_dataclass_args_schema: Option<PyObject>,
    visit_dataclass_schema: Option<PyObject>,
    visit_arguments_schema: Option<PyObject>,
    visit_call_schema: Option<PyObject>,
    visit_custom_error_schema: Option<PyObject>,
    visit_json_schema: Option<PyObject>,
    visit_url_schema: Option<PyObject>,
    visit_multi_host_url_schema: Option<PyObject>,
    visit_definitions_schema: Option<PyObject>,
    visit_definition_reference_schema: Option<PyObject>,
    visit_uuid_schema: Option<PyObject>,
    // ser schemas, see SerSchema in core_schema.py
    visit_plain_function_ser_schema: Option<PyObject>,
    visit_wrap_function_ser_schema: Option<PyObject>,
    visit_format_ser_schema: Option<PyObject>,
    visit_to_string_ser_schema: Option<PyObject>,
    visit_model_ser_schema: Option<PyObject>,
}

#[pymethods]
impl WalkCoreSchema {
    #[new]
    #[pyo3(signature = (visit_any_schema = None, visit_none_schema = None, visit_bool_schema = None, visit_int_schema = None, visit_float_schema = None, visit_decimal_schema = None, visit_string_schema = None, visit_bytes_schema = None, visit_date_schema = None, visit_time_schema = None, visit_datetime_schema = None, visit_timedelta_schema = None, visit_literal_schema = None, visit_is_instance_schema = None, visit_is_subclass_schema = None, visit_callable_schema = None, visit_list_schema = None, visit_tuple_positional_schema = None, visit_tuple_variable_schema = None, visit_set_schema = None, visit_frozenset_schema = None, visit_generator_schema = None, visit_dict_schema = None, visit_after_validator_function_schema = None, visit_before_validator_function_schema = None, visit_wrap_validator_function_schema = None, visit_plain_validator_function_schema = None, visit_with_default_schema = None, visit_nullable_schema = None, visit_union_schema = None, visit_tagged_union_schema = None, visit_chain_schema = None, visit_lax_or_strict_schema = None, visit_json_or_python_schema = None, visit_typed_dict_schema = None, visit_model_fields_schema = None, visit_model_schema = None, visit_dataclass_args_schema = None, visit_dataclass_schema = None, visit_arguments_schema = None, visit_call_schema = None, visit_custom_error_schema = None, visit_json_schema = None, visit_url_schema = None, visit_multi_host_url_schema = None, visit_definitions_schema = None, visit_definition_reference_schema = None, visit_uuid_schema = None, visit_plain_function_ser_schema = None, visit_wrap_function_ser_schema = None, visit_format_ser_schema = None, visit_to_string_ser_schema = None, visit_model_ser_schema = None))]
    fn new(
        visit_any_schema: Option<PyObject>,
        visit_none_schema: Option<PyObject>,
        visit_bool_schema: Option<PyObject>,
        visit_int_schema: Option<PyObject>,
        visit_float_schema: Option<PyObject>,
        visit_decimal_schema: Option<PyObject>,
        visit_string_schema: Option<PyObject>,
        visit_bytes_schema: Option<PyObject>,
        visit_date_schema: Option<PyObject>,
        visit_time_schema: Option<PyObject>,
        visit_datetime_schema: Option<PyObject>,
        visit_timedelta_schema: Option<PyObject>,
        visit_literal_schema: Option<PyObject>,
        visit_is_instance_schema: Option<PyObject>,
        visit_is_subclass_schema: Option<PyObject>,
        visit_callable_schema: Option<PyObject>,
        visit_list_schema: Option<PyObject>,
        visit_tuple_positional_schema: Option<PyObject>,
        visit_tuple_variable_schema: Option<PyObject>,
        visit_set_schema: Option<PyObject>,
        visit_frozenset_schema: Option<PyObject>,
        visit_generator_schema: Option<PyObject>,
        visit_dict_schema: Option<PyObject>,
        visit_after_validator_function_schema: Option<PyObject>,
        visit_before_validator_function_schema: Option<PyObject>,
        visit_wrap_validator_function_schema: Option<PyObject>,
        visit_plain_validator_function_schema: Option<PyObject>,
        visit_with_default_schema: Option<PyObject>,
        visit_nullable_schema: Option<PyObject>,
        visit_union_schema: Option<PyObject>,
        visit_tagged_union_schema: Option<PyObject>,
        visit_chain_schema: Option<PyObject>,
        visit_lax_or_strict_schema: Option<PyObject>,
        visit_json_or_python_schema: Option<PyObject>,
        visit_typed_dict_schema: Option<PyObject>,
        visit_model_fields_schema: Option<PyObject>,
        visit_model_schema: Option<PyObject>,
        visit_dataclass_args_schema: Option<PyObject>,
        visit_dataclass_schema: Option<PyObject>,
        visit_arguments_schema: Option<PyObject>,
        visit_call_schema: Option<PyObject>,
        visit_custom_error_schema: Option<PyObject>,
        visit_json_schema: Option<PyObject>,
        visit_url_schema: Option<PyObject>,
        visit_multi_host_url_schema: Option<PyObject>,
        visit_definitions_schema: Option<PyObject>,
        visit_definition_reference_schema: Option<PyObject>,
        visit_uuid_schema: Option<PyObject>,
        // ser schemas, see SerSchema in core_schema.py
        visit_plain_function_ser_schema: Option<PyObject>,
        visit_wrap_function_ser_schema: Option<PyObject>,
        visit_format_ser_schema: Option<PyObject>,
        visit_to_string_ser_schema: Option<PyObject>,
        visit_model_ser_schema: Option<PyObject>,
    ) -> Self {
        WalkCoreSchema {
            visit_any_schema,
            visit_none_schema,
            visit_bool_schema,
            visit_int_schema,
            visit_float_schema,
            visit_decimal_schema,
            visit_string_schema,
            visit_bytes_schema,
            visit_date_schema,
            visit_time_schema,
            visit_datetime_schema,
            visit_timedelta_schema,
            visit_literal_schema,
            visit_is_instance_schema,
            visit_is_subclass_schema,
            visit_callable_schema,
            visit_list_schema,
            visit_tuple_positional_schema,
            visit_tuple_variable_schema,
            visit_set_schema,
            visit_frozenset_schema,
            visit_generator_schema,
            visit_dict_schema,
            visit_after_validator_function_schema,
            visit_before_validator_function_schema,
            visit_wrap_validator_function_schema,
            visit_plain_validator_function_schema,
            visit_with_default_schema,
            visit_nullable_schema,
            visit_union_schema,
            visit_tagged_union_schema,
            visit_chain_schema,
            visit_lax_or_strict_schema,
            visit_json_or_python_schema,
            visit_typed_dict_schema,
            visit_model_fields_schema,
            visit_model_schema,
            visit_dataclass_args_schema,
            visit_dataclass_schema,
            visit_arguments_schema,
            visit_call_schema,
            visit_custom_error_schema,
            visit_json_schema,
            visit_url_schema,
            visit_multi_host_url_schema,
            visit_definitions_schema,
            visit_definition_reference_schema,
            visit_uuid_schema,
            // ser schemas, see SerSchema in core_schema.py
            visit_plain_function_ser_schema,
            visit_wrap_function_ser_schema,
            visit_format_ser_schema,
            visit_to_string_ser_schema,
            visit_model_ser_schema,
        }
    }

    fn walk<'s>(&self, py: Python<'s>, schema: &'s PyDict) -> PyResult<Py<PyDict>> {
        let schema_type: &str = schema.get_item("type")?.unwrap().extract()?;
        macro_rules! walk_core_schema {
            ($visit:expr, $handle:ident) => {
                match $visit {
                    Some(visit_any_schema) => visit_any_schema
                        .call1(
                            py,
                            (
                                schema,
                                &self
                                    .clone()
                                    .into_py(py)
                                    .getattr(py, intern!(py, stringify!($handle)))?,
                            ),
                        )?
                        .extract(py)?,
                    None => self.$handle(py, schema)?,
                }
            };
        }
        let schema = match schema_type {
            "any" => walk_core_schema!(&self.visit_any_schema, _handle_none_schema),
            "none" => walk_core_schema!(&self.visit_none_schema, _handle_none_schema),
            "bool" => walk_core_schema!(&self.visit_bool_schema, _handle_bool_schema),
            "int" => walk_core_schema!(&self.visit_int_schema, _handle_int_schema),
            "float" => walk_core_schema!(&self.visit_float_schema, _handle_float_schema),
            "decimal" => walk_core_schema!(&self.visit_decimal_schema, _handle_decimal_schema),
            "str" => walk_core_schema!(&self.visit_string_schema, _handle_string_schema),
            "bytes" => walk_core_schema!(&self.visit_bytes_schema, _handle_bytes_schema),
            "date" => walk_core_schema!(&self.visit_date_schema, _handle_date_schema),
            "time" => walk_core_schema!(&self.visit_time_schema, _handle_time_schema),
            "datetime" => walk_core_schema!(&self.visit_datetime_schema, _handle_datetime_schema),
            "timedelta" => walk_core_schema!(&self.visit_timedelta_schema, _handle_timedelta_schema),
            "literal" => walk_core_schema!(&self.visit_literal_schema, _handle_literal_schema),
            "is-instance" => walk_core_schema!(&self.visit_is_instance_schema, _handle_is_instance_schema),
            "is-subclass" => walk_core_schema!(&self.visit_is_subclass_schema, _handle_is_subclass_schema),
            "callable" => walk_core_schema!(&self.visit_callable_schema, _handle_callable_schema),
            "list" => walk_core_schema!(&self.visit_list_schema, _handle_list_schema),
            "tuple-positional" => {
                walk_core_schema!(&self.visit_tuple_positional_schema, _handle_tuple_positional_schema)
            }
            "tuple-variable" => walk_core_schema!(&self.visit_tuple_variable_schema, _handle_tuple_variable_schema),
            "set" => walk_core_schema!(&self.visit_set_schema, _handle_set_schema),
            "frozenset" => walk_core_schema!(&self.visit_frozenset_schema, _handle_frozenset_schema),
            "generator" => walk_core_schema!(&self.visit_generator_schema, _handle_generator_schema),
            "dict" => walk_core_schema!(&self.visit_dict_schema, _handle_dict_schema),
            "function-after" => walk_core_schema!(
                &self.visit_after_validator_function_schema,
                _handle_after_validator_function_schema
            ),
            "function-before" => walk_core_schema!(
                &self.visit_before_validator_function_schema,
                _handle_before_validator_function_schema
            ),
            "function-wrap" => walk_core_schema!(
                &self.visit_wrap_validator_function_schema,
                _handle_wrap_validator_function_schema
            ),
            "function-plain" => walk_core_schema!(
                &self.visit_plain_validator_function_schema,
                _handle_plain_validator_function_schema
            ),
            "default" => walk_core_schema!(&self.visit_with_default_schema, _handle_with_default_schema),
            "nullable" => walk_core_schema!(&self.visit_nullable_schema, _handle_nullable_schema),
            "union" => walk_core_schema!(&self.visit_union_schema, _handle_union_schema),
            "tagged-union" => walk_core_schema!(&self.visit_tagged_union_schema, _handle_tagged_union_schema),
            "chain" => walk_core_schema!(&self.visit_chain_schema, _handle_chain_schema),
            "lax-or-strict" => walk_core_schema!(&self.visit_lax_or_strict_schema, _handle_lax_or_strict_schema),
            "json-or-python" => walk_core_schema!(&self.visit_json_or_python_schema, _handle_json_or_python_schema),
            "typed-dict" => walk_core_schema!(&self.visit_typed_dict_schema, _handle_typed_dict_schema),
            "model-fields" => walk_core_schema!(&self.visit_model_fields_schema, _handle_model_fields_schema),
            "model" => walk_core_schema!(&self.visit_model_schema, _handle_model_schema),
            "dataclass-args" => walk_core_schema!(&self.visit_dataclass_args_schema, _handle_dataclass_args_schema),
            "dataclass" => walk_core_schema!(&self.visit_dataclass_schema, _handle_dataclass_schema),
            "arguments" => walk_core_schema!(&self.visit_arguments_schema, _handle_arguments_schema),
            "call" => walk_core_schema!(&self.visit_call_schema, _handle_call_schema),
            "custom-error" => walk_core_schema!(&self.visit_custom_error_schema, _handle_custom_error_schema),
            "json" => walk_core_schema!(&self.visit_json_schema, _handle_json_schema),
            "url" => walk_core_schema!(&self.visit_url_schema, _handle_url_schema),
            "multi-host-url" => walk_core_schema!(&self.visit_multi_host_url_schema, _handle_multi_host_url_schema),
            "definitions" => walk_core_schema!(&self.visit_definitions_schema, _handle_definitions_schema),
            "definition-ref" => walk_core_schema!(
                &self.visit_definition_reference_schema,
                _handle_definition_reference_schema
            ),
            "uuid" => walk_core_schema!(&self.visit_uuid_schema, _handle_uuid_schema),
            _ => return Err(PyTypeError::new_err(format!("Unknown schema type: {schema_type}"))),
        };
        Ok(schema)
    }

    fn _walk_ser_schema(&self, py: Python, ser_schema: &PyDict) -> PyResult<Py<PyDict>> {
        macro_rules! walk_ser_schema {
            ($visit:expr, $handle:ident) => {
                match $visit {
                    Some(visit_any_schema) => visit_any_schema
                        .call1(
                            py,
                            (
                                ser_schema,
                                &self
                                    .clone()
                                    .into_py(py)
                                    .getattr(py, intern!(py, stringify!($handle)))?,
                            ),
                        )?
                        .extract(py)?,
                    None => self.$handle(py, ser_schema)?,
                }
            };
        }
        let schema_type: &str = ser_schema.get_item("type")?.unwrap().extract()?;
        let schema = match schema_type {
            "none" | "int" | "bool" | "float" | "str" | "bytes" | "bytearray" | "list" | "tuple" | "set"
            | "frozenset" | "generator" | "dict" | "datetime" | "date" | "time" | "timedelta" | "url"
            | "multi-host-url" | "json" | "uuid" => {
                walk_ser_schema!(
                    &self.visit_plain_function_ser_schema,
                    _handle_plain_serializer_function_ser_schema
                )
            }
            "function-plain" => {
                walk_ser_schema!(
                    &self.visit_plain_function_ser_schema,
                    _handle_plain_serializer_function_ser_schema
                )
            }
            "function-wrap" => {
                walk_ser_schema!(
                    &self.visit_wrap_function_ser_schema,
                    _handle_wrap_serializer_function_ser_schema
                )
            }
            "format" => {
                walk_ser_schema!(&self.visit_format_ser_schema, _handle_format_ser_schema)
            }
            "to-string" => {
                walk_ser_schema!(&self.visit_to_string_ser_schema, _handle_to_string_ser_schema)
            }
            "model" => {
                walk_ser_schema!(&self.visit_model_ser_schema, _handle_model_ser_schema)
            }
            _ => return Err(PyTypeError::new_err(format!("Unknown ser schema type: {schema_type}"))),
        };
        Ok(schema)
    }

    // Check if there is a "serialization" key and if so call handle_ser_schema with it
    // and replace the result
    fn _check_ser_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let serialization_key = intern!(py, "serialization");
        let ser_schema: Option<&PyDict> = invert(schema.get_item(serialization_key)?.map(pyo3::PyAny::extract))?;
        match ser_schema {
            Some(ser_schema) => {
                let new_ser_schema = self._walk_ser_schema(py, ser_schema)?;
                schema.set_item("serialization", new_ser_schema)?;
            }
            None => {}
        }
        Ok(schema.into())
    }

    fn _handle_any_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_none_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_bool_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_int_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_float_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_decimal_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_string_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_bytes_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_date_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_time_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_datetime_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_timedelta_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_literal_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_is_instance_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_is_subclass_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_callable_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_list_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self._handle_items_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_tuple_positional_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let items_schema_key = intern!(py, "items_schema");
        let items_schema: Option<&PyList> = invert(schema.get_item(items_schema_key)?.map(pyo3::PyAny::extract))?;
        match items_schema {
            Some(items_schema) => {
                let new_items_schema = items_schema
                    .iter()
                    .map(|item_schema| self.walk(py, item_schema.extract()?))
                    .collect::<PyResult<Vec<Py<PyDict>>>>()?;
                schema.set_item(items_schema_key, new_items_schema)?;
            }
            None => {}
        }
        let schema = self._handle_extras_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_tuple_variable_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self._handle_items_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_set_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self._handle_items_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_frozenset_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self._handle_items_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_generator_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self._handle_items_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_dict_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let keys_schema: Option<&PyDict> = invert(schema.get_item("keys_schema")?.map(pyo3::PyAny::extract))?;
        match keys_schema {
            Some(keys_schema) => {
                let new_keys_schema = self.walk(py, keys_schema)?;
                schema.set_item("keys_schema", new_keys_schema)?;
            }
            None => {}
        }
        let values_schema: Option<&PyDict> = invert(schema.get_item("values_schema")?.map(pyo3::PyAny::extract))?;
        match values_schema {
            Some(values_schema) => {
                let new_values_schema = self.walk(py, values_schema)?;
                schema.set_item("values_schema", new_values_schema)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_after_validator_function_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_before_validator_function_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_wrap_validator_function_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_plain_validator_function_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_with_default_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_nullable_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_union_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let choices_key = intern!(py, "choices");
        let choices: Option<&PyList> = invert(schema.get_item(choices_key)?.map(pyo3::PyAny::extract))?;
        match choices {
            Some(choices) => {
                let new_choices = choices
                    .iter()
                    .map(|choice| self.walk(py, choice.extract()?))
                    .collect::<PyResult<Vec<Py<PyDict>>>>()?;
                schema.set_item(choices_key, new_choices)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_tagged_union_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let choices_key = intern!(py, "choices");
        let choices: Option<&PyDict> = invert(schema.get_item(choices_key)?.map(pyo3::PyAny::extract))?;
        match choices {
            Some(choices) => {
                let new_choices = choices
                    .iter()
                    .map(|(k, v)| {
                        let new_v = self.walk(py, v.extract()?)?;
                        Ok((k, new_v.into_ref(py).into()))
                    })
                    .collect::<PyResult<Vec<(&PyAny, &PyAny)>>>()?;
                schema.set_item(choices_key, PyDict::from_sequence(py, new_choices.into_py(py))?)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_chain_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let steps_key = intern!(py, "steps");
        let steps: Option<&PyList> = invert(schema.get_item(steps_key)?.map(pyo3::PyAny::extract))?;
        match steps {
            Some(steps) => {
                let new_steps = steps
                    .iter()
                    .map(|step| self.walk(py, step.extract()?))
                    .collect::<PyResult<Vec<Py<PyDict>>>>()?;
                schema.set_item(steps_key, new_steps)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_lax_or_strict_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let lax_schema_key = intern!(py, "lax_schema");
        let lax_schema: Option<&PyDict> = invert(schema.get_item(lax_schema_key)?.map(pyo3::PyAny::extract))?;
        match lax_schema {
            Some(lax_schema) => {
                let new_lax_schema = self.walk(py, lax_schema)?;
                schema.set_item(lax_schema_key, new_lax_schema)?;
            }
            None => {}
        }
        let strict_schema_key = intern!(py, "strict_schema");
        let strict_schema: Option<&PyDict> = invert(schema.get_item(strict_schema_key)?.map(pyo3::PyAny::extract))?;
        match strict_schema {
            Some(strict_schema) => {
                let new_strict_schema = self.walk(py, strict_schema)?;
                schema.set_item(strict_schema_key, new_strict_schema)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_json_or_python_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let json_schema_key = intern!(py, "json_schema");
        let json_schema: Option<&PyDict> = invert(schema.get_item(json_schema_key)?.map(pyo3::PyAny::extract))?;
        match json_schema {
            Some(json_schema) => {
                let new_json_schema = self.walk(py, json_schema)?;
                schema.set_item(json_schema_key, new_json_schema)?;
            }
            None => {}
        }
        let python_schema_key = intern!(py, "python_schema");
        let python_schema: Option<&PyDict> = invert(schema.get_item(python_schema_key)?.map(pyo3::PyAny::extract))?;
        match python_schema {
            Some(python_schema) => {
                let new_python_schema = self.walk(py, python_schema)?;
                schema.set_item(python_schema_key, new_python_schema)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_typed_dict_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        match fields {
            Some(fields) => {
                let new_fields = fields
                    .iter()
                    .map(|(k, v)| {
                        // v is of type TypedDictField in core_schema.py
                        let typed_dict_field: &PyDict = v.extract()?;
                        let schema: &PyDict = typed_dict_field
                            .get_item("schema")
                            .ok()
                            .flatten()
                            .ok_or_else(|| PyTypeError::new_err("Missing schema in TypedDictField"))?
                            .extract()?;
                        let new_schema = self.walk(py, schema)?;
                        typed_dict_field.set_item("schema", new_schema)?;
                        Ok((k, v))
                    })
                    .collect::<PyResult<Vec<(&PyAny, &PyAny)>>>()?;
                schema.set_item(fields_key, PyDict::from_sequence(py, new_fields.into_py(py))?)?;
            }
            None => {}
        }
        let schema = self._handle_extras_schema(py, schema)?.into_ref(py).extract()?;
        let schema = self._handle_computed_fields_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_model_fields_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        match fields {
            Some(fields) => {
                let new_fields = fields
                    .iter()
                    .map(|(k, v)| {
                        let new_v = self.walk(py, v.extract()?)?;
                        Ok((k, new_v))
                    })
                    .collect::<PyResult<Vec<(&PyAny, Py<PyDict>)>>>()?;
                schema.set_item(fields_key, new_fields)?;
            }
            None => {}
        }
        let schema = self._handle_extras_schema(py, schema)?.into_ref(py).extract()?;
        let schema = self._handle_computed_fields_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_model_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        match fields {
            Some(fields) => {
                let new_fields = fields
                    .iter()
                    .map(|(k, v)| {
                        let new_v = self.walk(py, v.extract()?)?;
                        Ok((k, new_v))
                    })
                    .collect::<PyResult<Vec<(&PyAny, Py<PyDict>)>>>()?;
                schema.set_item(fields_key, new_fields)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_dataclass_args_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let schema_key = intern!(py, "schema");
        let fields: Option<&PyList> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        match fields {
            Some(fields) => {
                for v in fields {
                    let dataclass_field: &PyDict = v.extract()?;
                    let dataclass_field_schema: &PyDict =
                        invert(dataclass_field.get_item(schema_key)?.map(pyo3::PyAny::extract))?.unwrap();
                    let new_dataclass_field_schema = self.walk(py, dataclass_field_schema)?;
                    dataclass_field.set_item(schema_key, new_dataclass_field_schema)?;
                }
            }
            None => {}
        }
        let schema = self._handle_computed_fields_schema(py, schema)?.into_ref(py).extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_dataclass_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_arguments_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let var_args_schema_key = intern!(py, "var_args_schema");
        let var_kwargs_schema_key = intern!(py, "var_kwargs_schema");
        let var_args_schema: Option<&PyDict> = invert(schema.get_item(var_args_schema_key)?.map(pyo3::PyAny::extract))?;
        match var_args_schema {
            Some(var_args_schema) => {
                let new_var_args_schema = self.walk(py, var_args_schema)?;
                schema.set_item(var_args_schema_key, new_var_args_schema)?;
            }
            None => {}
        }
        let var_kwargs_schema: Option<&PyDict> =
            invert(schema.get_item(var_kwargs_schema_key)?.map(pyo3::PyAny::extract))?;
        match var_kwargs_schema {
            Some(var_kwargs_schema) => {
                let new_var_kwargs_schema = self.walk(py, var_kwargs_schema)?;
                schema.set_item(var_kwargs_schema_key, new_var_kwargs_schema)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_call_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let arguments_schema_key = intern!(py, "arguments_schema");
        let return_schema_key = intern!(py, "return_schema");
        let arguments_schema: Option<&PyDict> =
            invert(schema.get_item(arguments_schema_key)?.map(pyo3::PyAny::extract))?;
        match arguments_schema {
            Some(arguments_schema) => {
                let new_arguments_schema = self.walk(py, arguments_schema)?;
                schema.set_item(arguments_schema_key, new_arguments_schema)?;
            }
            None => {}
        }
        let return_schema: Option<&PyDict> = invert(schema.get_item(return_schema_key)?.map(pyo3::PyAny::extract))?;
        match return_schema {
            Some(return_schema) => {
                let new_return_schema = self.walk(py, return_schema)?;
                schema.set_item(return_schema_key, new_return_schema)?;
            }
            None => {}
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_custom_error_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_json_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_url_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_multi_host_url_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_definitions_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let definitions_key = intern!(py, "definitions");
        let definitions: Option<&PyList> = invert(schema.get_item(definitions_key)?.map(pyo3::PyAny::extract))?;
        match definitions {
            Some(definitions) => {
                let new_definitions = definitions
                    .iter()
                    .map(|definition| self.walk(py, definition.extract()?))
                    .collect::<PyResult<Vec<Py<PyDict>>>>()?;
                schema.set_item(definitions_key, new_definitions)?;
            }
            None => {}
        }
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_definition_reference_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_uuid_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._check_ser_schema(py, schema)
    }

    fn _handle_simple_ser_schema(&self, _py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        Ok(schema.into())
    }

    fn _handle_plain_serializer_function_ser_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._handle_inner_schema(py, schema, intern!(py, "return_schema"))
    }

    fn _handle_wrap_serializer_function_ser_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema = self
            ._handle_inner_schema(py, schema, intern!(py, "schema"))?
            .into_ref(py)
            .extract()?;
        self._handle_inner_schema(py, schema, intern!(py, "return_schema"))
    }

    fn _handle_format_ser_schema(&self, _py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        Ok(schema.into())
    }

    fn _handle_to_string_ser_schema(&self, _py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        Ok(schema.into())
    }

    fn _handle_model_ser_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        self._handle_inner_schema(py, schema, intern!(py, "schema"))
    }

    // Handle a dict where there may be a `"schema": CoreSchema` key
    fn _handle_inner_schema(&self, py: Python, schema: &PyDict, schema_key: &PyString) -> PyResult<Py<PyDict>> {
        let inner_schema: Option<&PyDict> = invert(schema.get_item(schema_key)?.map(pyo3::PyAny::extract))?;
        match inner_schema {
            Some(inner_schema) => {
                let new_inner_schema = self.walk(py, inner_schema)?;
                schema.set_item(schema_key, new_inner_schema)?;
            }
            None => {}
        }
        Ok(schema.into())
    }

    // Handle a dict where there may be a `"items_schema": CoreSchema` key
    fn _handle_items_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let items_schema_key = intern!(py, "items_schema");
        let items_schema: Option<&PyDict> = invert(schema.get_item(items_schema_key)?.map(pyo3::PyAny::extract))?;
        match items_schema {
            Some(items_schema) => {
                let new_items_schema = self.walk(py, items_schema)?;
                schema.set_item(items_schema_key, new_items_schema)?;
            }
            None => {}
        }
        Ok(schema.into())
    }

    fn _handle_computed_fields_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let computed_fields_key = intern!(py, "computed_fields");
        let computed_fields: Option<&PyList> = invert(schema.get_item(computed_fields_key)?.map(pyo3::PyAny::extract))?;
        match computed_fields {
            Some(computed_fields) => {
                let schema_key = intern!(py, "schema");
                let new_computed_fields = computed_fields
                    .iter()
                    .map(|computed_field| {
                        let computed_field_schema: &PyDict = computed_field.extract()?;
                        self._handle_inner_schema(py, computed_field_schema, schema_key)
                            .map(|v| v.into())
                    })
                    .collect::<PyResult<Vec<PyObject>>>()?;
                schema.set_item(computed_fields_key, new_computed_fields)?;
            }
            None => {}
        }
        Ok(schema.into())
    }

    fn _handle_extras_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let extras_schema_key = intern!(py, "extras_schema");
        let extras_schema: Option<&PyDict> = invert(schema.get_item(extras_schema_key)?.map(pyo3::PyAny::extract))?;
        match extras_schema {
            Some(extras_schema) => {
                let new_extras_schema = self.walk(py, extras_schema)?;
                schema.set_item(extras_schema_key, new_extras_schema)?;
            }
            None => {}
        }
        Ok(schema.into())
    }
}

fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}
