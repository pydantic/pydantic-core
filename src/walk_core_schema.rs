use pyo3::exceptions::PyTypeError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

#[pyclass(subclass, module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct WalkCoreSchema {
    visit_core_schema: Option<FilterCallable>,
    visit_ser_schema: Option<FilterCallable>,
}

#[pymethods]
impl WalkCoreSchema {
    #[new]
    #[pyo3(signature = (visit_core_schema = None, visit_ser_schema = None))]
    fn new(visit_core_schema: Option<FilterCallable>, visit_ser_schema: Option<FilterCallable>) -> Self {
        WalkCoreSchema {
            visit_core_schema,
            visit_ser_schema,
        }
    }

    fn walk<'s>(&self, py: Python<'s>, schema: &'s PyDict) -> PyResult<Py<PyDict>> {
        match &self.visit_core_schema {
            Some(visit_core_schema) => {
                if visit_core_schema.matches(py, schema)? {
                    let call_next = self
                        .clone()
                        .into_py(py)
                        .getattr(py, intern!(py, "_walk_core_schema"))?
                        .clone();
                    visit_core_schema.call(py, schema, call_next)?.extract(py)
                } else {
                    self._walk_core_schema(py, schema)
                }
            }
            None => self._walk_core_schema(py, schema),
        }
    }

    fn _walk_core_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema_type: &str = schema.get_item("type")?.unwrap().extract()?;
        match schema_type {
            "any" => self._handle_any_schema(py, schema),
            "none" => self._handle_none_schema(py, schema),
            "bool" => self._handle_bool_schema(py, schema),
            "int" => self._handle_int_schema(py, schema),
            "float" => self._handle_float_schema(py, schema),
            "decimal" => self._handle_decimal_schema(py, schema),
            "str" => self._handle_string_schema(py, schema),
            "bytes" => self._handle_bytes_schema(py, schema),
            "date" => self._handle_date_schema(py, schema),
            "time" => self._handle_time_schema(py, schema),
            "datetime" => self._handle_datetime_schema(py, schema),
            "timedelta" => self._handle_timedelta_schema(py, schema),
            "literal" => self._handle_literal_schema(py, schema),
            "is-instance" => self._handle_is_instance_schema(py, schema),
            "is-subclass" => self._handle_is_subclass_schema(py, schema),
            "callable" => self._handle_callable_schema(py, schema),
            "list" => self._handle_list_schema(py, schema),
            "tuple-positional" => self._handle_tuple_positional_schema(py, schema),
            "tuple-variable" => self._handle_tuple_variable_schema(py, schema),
            "set" => self._handle_set_schema(py, schema),
            "frozenset" => self._handle_frozenset_schema(py, schema),
            "generator" => self._handle_generator_schema(py, schema),
            "dict" => self._handle_dict_schema(py, schema),
            "function-after" => self._handle_after_validator_function_schema(py, schema),
            "function-before" => self._handle_before_validator_function_schema(py, schema),
            "function-wrap" => self._handle_wrap_validator_function_schema(py, schema),
            "function-plain" => self._handle_plain_validator_function_schema(py, schema),
            "default" => self._handle_with_default_schema(py, schema),
            "nullable" => self._handle_nullable_schema(py, schema),
            "union" => self._handle_union_schema(py, schema),
            "tagged-union" => self._handle_tagged_union_schema(py, schema),
            "chain" => self._handle_chain_schema(py, schema),
            "lax-or-strict" => self._handle_lax_or_strict_schema(py, schema),
            "json-or-python" => self._handle_json_or_python_schema(py, schema),
            "typed-dict" => self._handle_typed_dict_schema(py, schema),
            "model-fields" => self._handle_model_fields_schema(py, schema),
            "model" => self._handle_model_schema(py, schema),
            "dataclass-args" => self._handle_dataclass_args_schema(py, schema),
            "dataclass" => self._handle_dataclass_schema(py, schema),
            "arguments" => self._handle_arguments_schema(py, schema),
            "call" => self._handle_call_schema(py, schema),
            "custom-error" => self._handle_custom_error_schema(py, schema),
            "json" => self._handle_json_schema(py, schema),
            "url" => self._handle_url_schema(py, schema),
            "multi-host-url" => self._handle_multi_host_url_schema(py, schema),
            "definitions" => self._handle_definitions_schema(py, schema),
            "definition-ref" => self._handle_definition_reference_schema(py, schema),
            "uuid" => self._handle_uuid_schema(py, schema),
            _ => Err(PyTypeError::new_err(format!("Unknown schema type: {schema_type}"))),
        }
    }

    fn _walk_ser_schema(&self, py: Python, ser_schema: &PyDict) -> PyResult<Py<PyDict>> {
        let schema_type: &str = ser_schema.get_item("type")?.unwrap().extract()?;
        match schema_type {
            "none" | "int" | "bool" | "float" | "str" | "bytes" | "bytearray" | "list" | "tuple" | "set"
            | "frozenset" | "generator" | "dict" | "datetime" | "date" | "time" | "timedelta" | "url"
            | "multi-host-url" | "json" | "uuid" => self._handle_simple_ser_schema(py, ser_schema),
            "function-plain" => self._handle_plain_serializer_function_ser_schema(py, ser_schema),
            "function-wrap" => self._handle_wrap_serializer_function_ser_schema(py, ser_schema),
            "format" => self._handle_format_ser_schema(py, ser_schema),
            "to-string" => self._handle_to_string_ser_schema(py, ser_schema),
            "model" => self._handle_model_ser_schema(py, ser_schema),
            _ => Err(PyTypeError::new_err(format!("Unknown ser schema type: {schema_type}"))),
        }
    }

    // Check if there is a "serialization" key and if so handle it
    // and replace the result
    fn _check_ser_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let serialization_key = intern!(py, "serialization");
        let ser_schema: Option<&PyDict> = invert(schema.get_item(serialization_key)?.map(pyo3::PyAny::extract))?;
        if let Some(ser_schema) = ser_schema {
            if let Some(visit_ser_schema) = &self.visit_ser_schema {
                if visit_ser_schema.matches(py, ser_schema)? {
                    let call_next = self
                        .clone()
                        .into_py(py)
                        .getattr(py, intern!(py, "_walk_ser_schema"))?
                        .clone();
                    let new_ser_schema = visit_ser_schema.call(py, ser_schema, call_next)?;
                    schema.set_item(serialization_key, new_ser_schema)?;
                } else {
                    let new_ser_schema = self._walk_ser_schema(py, ser_schema)?;
                    schema.set_item(serialization_key, new_ser_schema)?;
                }
            }
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
        if let Some(items_schema) = items_schema {
            let new_items_schema = items_schema
                .iter()
                .map(|item_schema| self.walk(py, item_schema.extract()?))
                .collect::<PyResult<Vec<Py<PyDict>>>>()?;
            schema.set_item(items_schema_key, new_items_schema)?;
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
        if let Some(keys_schema) = keys_schema {
            let new_keys_schema = self.walk(py, keys_schema)?;
            schema.set_item("keys_schema", new_keys_schema)?;
        }
        let values_schema: Option<&PyDict> = invert(schema.get_item("values_schema")?.map(pyo3::PyAny::extract))?;
        if let Some(values_schema) = values_schema {
            let new_values_schema = self.walk(py, values_schema)?;
            schema.set_item("values_schema", new_values_schema)?;
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
        if let Some(choices) = choices {
            let new_choices = choices
                .iter()
                .map(|choice| self.walk(py, choice.extract()?))
                .collect::<PyResult<Vec<Py<PyDict>>>>()?;
            schema.set_item(choices_key, new_choices)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_tagged_union_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let choices_key = intern!(py, "choices");
        let choices: Option<&PyDict> = invert(schema.get_item(choices_key)?.map(pyo3::PyAny::extract))?;
        if let Some(choices) = choices {
            let new_choices = choices.iter().map(|(k, v)| {
                let new_v = self.walk(py, v.extract()?);
                Ok((k, new_v?))
            });
            schema.set_item(choices_key, py_dict_from_iterator(py, new_choices)?)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_chain_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let steps_key = intern!(py, "steps");
        let steps: Option<&PyList> = invert(schema.get_item(steps_key)?.map(pyo3::PyAny::extract))?;
        if let Some(steps) = steps {
            let new_steps = steps
                .iter()
                .map(|step| self.walk(py, step.extract()?))
                .collect::<PyResult<Vec<Py<PyDict>>>>()?;
            schema.set_item(steps_key, new_steps)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_lax_or_strict_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let lax_schema_key = intern!(py, "lax_schema");
        let lax_schema: Option<&PyDict> = invert(schema.get_item(lax_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(lax_schema) = lax_schema {
            let new_lax_schema = self.walk(py, lax_schema)?;
            schema.set_item(lax_schema_key, new_lax_schema)?;
        }
        let strict_schema_key = intern!(py, "strict_schema");
        let strict_schema: Option<&PyDict> = invert(schema.get_item(strict_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(strict_schema) = strict_schema {
            let new_strict_schema = self.walk(py, strict_schema)?;
            schema.set_item(strict_schema_key, new_strict_schema)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_json_or_python_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let json_schema_key = intern!(py, "json_schema");
        let json_schema: Option<&PyDict> = invert(schema.get_item(json_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(json_schema) = json_schema {
            let new_json_schema = self.walk(py, json_schema)?;
            schema.set_item(json_schema_key, new_json_schema)?;
        }
        let python_schema_key = intern!(py, "python_schema");
        let python_schema: Option<&PyDict> = invert(schema.get_item(python_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(python_schema) = python_schema {
            let new_python_schema = self.walk(py, python_schema)?;
            schema.set_item(python_schema_key, new_python_schema)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_typed_dict_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        if let Some(fields) = fields {
            let new_fields = fields.iter().map(|(k, v)| {
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
            });
            schema.set_item(fields_key, py_dict_from_iterator(py, new_fields)?)?;
        }
        let schema = self._handle_extras_schema(py, schema)?.into_ref(py).extract()?;
        let schema = self
            ._handle_computed_fields_schema(py, schema)?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_model_fields_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        if let Some(fields) = fields {
            let new_fields = fields
                .iter()
                .map(|(k, v)| {
                    let new_v = self.walk(py, v.extract()?)?;
                    Ok((k, new_v))
                })
                .collect::<PyResult<Vec<(&PyAny, Py<PyDict>)>>>()?;
            schema.set_item(fields_key, new_fields)?;
        }
        let schema = self._handle_extras_schema(py, schema)?.into_ref(py).extract()?;
        let schema = self
            ._handle_computed_fields_schema(py, schema)?
            .into_ref(py)
            .extract()?;
        self._check_ser_schema(py, schema)
    }

    fn _handle_model_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let fields: Option<&PyDict> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        if let Some(fields) = fields {
            let new_fields = fields
                .iter()
                .map(|(k, v)| {
                    let new_v = self.walk(py, v.extract()?)?;
                    Ok((k, new_v))
                })
                .collect::<PyResult<Vec<(&PyAny, Py<PyDict>)>>>()?;
            schema.set_item(fields_key, new_fields)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_dataclass_args_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let fields_key = intern!(py, "fields");
        let schema_key = intern!(py, "schema");
        let fields: Option<&PyList> = invert(schema.get_item(fields_key)?.map(pyo3::PyAny::extract))?;
        if let Some(fields) = fields {
            for v in fields {
                let dataclass_field: &PyDict = v.extract()?;
                let dataclass_field_schema: &PyDict =
                    invert(dataclass_field.get_item(schema_key)?.map(pyo3::PyAny::extract))?.unwrap();
                let new_dataclass_field_schema = self.walk(py, dataclass_field_schema)?;
                dataclass_field.set_item(schema_key, new_dataclass_field_schema)?;
            }
        }
        let schema = self
            ._handle_computed_fields_schema(py, schema)?
            .into_ref(py)
            .extract()?;
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
        if let Some(var_args_schema) = var_args_schema {
            let new_var_args_schema = self.walk(py, var_args_schema)?;
            schema.set_item(var_args_schema_key, new_var_args_schema)?;
        }
        let var_kwargs_schema: Option<&PyDict> =
            invert(schema.get_item(var_kwargs_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(var_kwargs_schema) = var_kwargs_schema {
            let new_var_kwargs_schema = self.walk(py, var_kwargs_schema)?;
            schema.set_item(var_kwargs_schema_key, new_var_kwargs_schema)?;
        }
        self._check_ser_schema(py, schema)
    }

    fn _handle_call_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let arguments_schema_key = intern!(py, "arguments_schema");
        let return_schema_key = intern!(py, "return_schema");
        let arguments_schema: Option<&PyDict> =
            invert(schema.get_item(arguments_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(arguments_schema) = arguments_schema {
            let new_arguments_schema = self.walk(py, arguments_schema)?;
            schema.set_item(arguments_schema_key, new_arguments_schema)?;
        }
        let return_schema: Option<&PyDict> = invert(schema.get_item(return_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(return_schema) = return_schema {
            let new_return_schema = self.walk(py, return_schema)?;
            schema.set_item(return_schema_key, new_return_schema)?;
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
        if let Some(definitions) = definitions {
            let new_definitions = definitions
                .iter()
                .map(|definition| self.walk(py, definition.extract()?))
                .collect::<PyResult<Vec<Py<PyDict>>>>()?;
            schema.set_item(definitions_key, new_definitions)?;
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
        if let Some(inner_schema) = inner_schema {
            let new_inner_schema = self.walk(py, inner_schema)?;
            schema.set_item(schema_key, new_inner_schema)?;
        }
        Ok(schema.into())
    }

    // Handle a dict where there may be a `"items_schema": CoreSchema` key
    fn _handle_items_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let items_schema_key = intern!(py, "items_schema");
        let items_schema: Option<&PyDict> = invert(schema.get_item(items_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(items_schema) = items_schema {
            let new_items_schema = self.walk(py, items_schema)?;
            schema.set_item(items_schema_key, new_items_schema)?;
        }
        Ok(schema.into())
    }

    fn _handle_computed_fields_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let computed_fields_key = intern!(py, "computed_fields");
        let computed_fields: Option<&PyList> = invert(schema.get_item(computed_fields_key)?.map(pyo3::PyAny::extract))?;
        if let Some(computed_fields) = computed_fields {
            let schema_key = intern!(py, "schema");
            let new_computed_fields = computed_fields
                .iter()
                .map(|computed_field| {
                    let computed_field_schema: &PyDict = computed_field.extract()?;
                    self._handle_inner_schema(py, computed_field_schema, schema_key)
                })
                .collect::<PyResult<Vec<Py<PyDict>>>>()?;
            schema.set_item(computed_fields_key, new_computed_fields)?;
        }
        Ok(schema.into())
    }

    fn _handle_extras_schema(&self, py: Python, schema: &PyDict) -> PyResult<Py<PyDict>> {
        let extras_schema_key = intern!(py, "extras_schema");
        let extras_schema: Option<&PyDict> = invert(schema.get_item(extras_schema_key)?.map(pyo3::PyAny::extract))?;
        if let Some(extras_schema) = extras_schema {
            let new_extras_schema = self.walk(py, extras_schema)?;
            schema.set_item(extras_schema_key, new_extras_schema)?;
        }
        Ok(schema.into())
    }
}

fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}

#[derive(Debug, Clone)]
enum Filter {
    HasRef,
    HasType { type_: String },
    Python { predicate: PyObject },
    And { left: Box<Filter>, right: Box<Filter> },
    Or { left: Box<Filter>, right: Box<Filter> },
}

impl Filter {
    fn matches(&self, py: Python, schema: &PyDict) -> PyResult<bool> {
        match self {
            Filter::HasRef => {
                let ref_ = schema.get_item("ref")?;
                Ok(ref_.is_some())
            }
            Filter::HasType { type_ } => {
                if let Some(schema_type) = invert(schema.get_item("type")?.map(pyo3::PyAny::extract::<&str>))? {
                    Ok(schema_type == type_)
                } else {
                    Ok(false)
                }
            }
            Filter::Python { predicate } => {
                let result: bool = predicate.call1(py, (schema,))?.extract(py)?;
                Ok(result)
            }
            Filter::And { left, right } => Ok(left.matches(py, schema)? && right.matches(py, schema)?),
            Filter::Or { left, right } => Ok(left.matches(py, schema)? || right.matches(py, schema)?),
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(module = "pydantic_core._pydantic_core")]
pub struct WalkCoreSchemaFilterBuilder {
    filter: Filter,
}

#[pymethods]
impl WalkCoreSchemaFilterBuilder {
    #[staticmethod]
    fn has_ref() -> Self {
        WalkCoreSchemaFilterBuilder { filter: Filter::HasRef }
    }

    #[staticmethod]
    #[pyo3(text_signature = "(type)")]
    fn has_type(type_: String) -> Self {
        WalkCoreSchemaFilterBuilder {
            filter: Filter::HasType { type_ },
        }
    }

    #[staticmethod]
    fn predicate(predicate: PyObject) -> Self {
        WalkCoreSchemaFilterBuilder {
            filter: Filter::Python { predicate },
        }
    }

    fn __and__(&self, other: WalkCoreSchemaFilterBuilder) -> Self {
        WalkCoreSchemaFilterBuilder {
            filter: Filter::And {
                left: Box::new(self.filter.clone()),
                right: Box::new(other.filter.clone()),
            },
        }
    }

    fn __or__(&self, other: WalkCoreSchemaFilterBuilder) -> Self {
        WalkCoreSchemaFilterBuilder {
            filter: Filter::Or {
                left: Box::new(self.filter.clone()),
                right: Box::new(other.filter.clone()),
            },
        }
    }

    fn build(&self, func: PyObject) -> FilterCallable {
        FilterCallable {
            filter: self.filter.clone(),
            func,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(module = "pydantic_core._pydantic_core")]
struct FilterCallable {
    filter: Filter,
    func: PyObject,
}

impl FilterCallable {
    fn matches(&self, py: Python, schema: &PyDict) -> PyResult<bool> {
        self.filter.matches(py, schema)
    }

    fn call(&self, py: Python, schema: &PyDict, call_next: PyObject) -> PyResult<Py<PyDict>> {
        self.func.call1(py, (schema, call_next))?.extract(py)
    }
}

fn py_dict_from_iterator<K: ToPyObject, V: ToPyObject>(
    py: Python,
    iterator: impl IntoIterator<Item = PyResult<(K, V)>>,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    for item in iterator {
        let (k, v) = item?;
        dict.set_item(k, v)?;
    }
    Ok(dict.into())
}
