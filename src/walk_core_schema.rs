use pyo3::exceptions::PyKeyError;
use pyo3::exceptions::PyTypeError;
use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::types::{PyDict, PyList, PyString};

#[pyclass(subclass, frozen, module = "pydantic_core._pydantic_core")]
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

    fn walk<'s>(&self, schema: &Py2<'s, PyDict>) -> PyResult<Py2<'s, PyDict>> {
        match &self.visit_core_schema {
            Some(visit_core_schema) => {
                let py = schema.py();
                if visit_core_schema.matches(schema)? {
                    let call_next = self
                        .clone()
                        .into_py(py)
                        .getattr(py, intern2!(schema.py(), "_walk_core_schema"))?;
                    visit_core_schema.call(&schema.copy()?, call_next)
                } else {
                    self._walk_core_schema(schema)
                }
            }
            None => self._walk_core_schema(schema),
        }
    }

    fn _walk_core_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        // TODO: can we remove this copy by keeping track of when we hit a filter
        // (schemas con only get modified if they hit a filter)
        let schema = &schema.copy()?;
        let schema_type: Py2<'_, PyString> = schema.get_item("type")?.unwrap().downcast_into()?;
        let schema_type = schema_type.to_str()?;
        match schema_type {
            "any" => self._handle_any_schema(schema),
            "none" => self._handle_none_schema(schema),
            "bool" => self._handle_bool_schema(schema),
            "int" => self._handle_int_schema(schema),
            "float" => self._handle_float_schema(schema),
            "decimal" => self._handle_decimal_schema(schema),
            "str" => self._handle_string_schema(schema),
            "bytes" => self._handle_bytes_schema(schema),
            "date" => self._handle_date_schema(schema),
            "time" => self._handle_time_schema(schema),
            "datetime" => self._handle_datetime_schema(schema),
            "timedelta" => self._handle_timedelta_schema(schema),
            "literal" => self._handle_literal_schema(schema),
            "is-instance" => self._handle_is_instance_schema(schema),
            "is-subclass" => self._handle_is_subclass_schema(schema),
            "callable" => self._handle_callable_schema(schema),
            "list" => self._handle_list_schema(schema),
            "tuple-positional" => self._handle_tuple_positional_schema(schema),
            "tuple-variable" => self._handle_tuple_variable_schema(schema),
            "set" => self._handle_set_schema(schema),
            "frozenset" => self._handle_frozenset_schema(schema),
            "generator" => self._handle_generator_schema(schema),
            "dict" => self._handle_dict_schema(schema),
            "function-after" => self._handle_after_validator_function_schema(schema),
            "function-before" => self._handle_before_validator_function_schema(schema),
            "function-wrap" => self._handle_wrap_validator_function_schema(schema),
            "function-plain" => self._handle_plain_validator_function_schema(schema),
            "default" => self._handle_with_default_schema(schema),
            "nullable" => self._handle_nullable_schema(schema),
            "union" => self._handle_union_schema(schema),
            "tagged-union" => self._handle_tagged_union_schema(schema),
            "chain" => self._handle_chain_schema(schema),
            "lax-or-strict" => self._handle_lax_or_strict_schema(schema),
            "json-or-python" => self._handle_json_or_python_schema(schema),
            "typed-dict" => self._handle_typed_dict_schema(schema),
            "model-fields" => self._handle_model_fields_schema(schema),
            "model-field" => self._handle_model_field_schema(schema),
            "model" => self._handle_model_schema(schema),
            "dataclass-args" => self._handle_dataclass_args_schema(schema),
            "dataclass" => self._handle_dataclass_schema(schema),
            "arguments" => self._handle_arguments_schema(schema),
            "call" => self._handle_call_schema(schema),
            "custom-error" => self._handle_custom_error_schema(schema),
            "json" => self._handle_json_schema(schema),
            "url" => self._handle_url_schema(schema),
            "multi-host-url" => self._handle_multi_host_url_schema(schema),
            "definitions" => self._handle_definitions_schema(schema),
            "definition-ref" => self._handle_definition_reference_schema(schema),
            "uuid" => self._handle_uuid_schema(schema),
            _ => Err(PyTypeError::new_err(format!("Unknown schema type: {schema_type}"))),
        }
    }

    fn _walk_ser_schema<'py>(&self, ser_schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        // TODO: can we remove this copy by keeping track of when we hit a filter
        // (schemas con only get modified if they hit a filter)
        let ser_schema = &ser_schema.copy()?;
        let schema_type = ser_schema
            .get_item("type")?
            .ok_or_else(|| PyKeyError::new_err("type"))?
            .downcast_into::<PyString>()?;
        let schema_type = schema_type.to_str()?;

        match schema_type {
            "none" | "int" | "bool" | "float" | "str" | "bytes" | "bytearray" | "list" | "tuple" | "set"
            | "frozenset" | "generator" | "dict" | "datetime" | "date" | "time" | "timedelta" | "url"
            | "multi-host-url" | "json" | "uuid" => self._handle_simple_ser_schema(ser_schema),
            "function-plain" => self._handle_plain_serializer_function_ser_schema(ser_schema),
            "function-wrap" => self._handle_wrap_serializer_function_ser_schema(ser_schema),
            "format" => self._handle_format_ser_schema(ser_schema),
            "to-string" => self._handle_to_string_ser_schema(ser_schema),
            "model" => self._handle_model_ser_schema(ser_schema),
            _ => Err(PyTypeError::new_err(format!("Unknown ser schema type: {schema_type}"))),
        }
    }

    // Check if there is a "serialization" key and if so handle it
    // and replace the result
    fn _check_ser_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let serialization_key = intern2!(schema.py(), "serialization");
        let ser_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(serialization_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(ser_schema) = &ser_schema {
            let py = schema.py();
            if let Some(visit_ser_schema) = &self.visit_ser_schema {
                if visit_ser_schema.matches(ser_schema)? {
                    let call_next = self
                        .clone()
                        .into_py(py)
                        .getattr(py, intern2!(schema.py(), "_walk_ser_schema"))?
                        .clone();
                    let new_ser_schema = visit_ser_schema.call(ser_schema, call_next)?;
                    schema.set_item(serialization_key, new_ser_schema)?;
                } else {
                    let new_ser_schema = self._walk_ser_schema(ser_schema)?;
                    schema.set_item(serialization_key, new_ser_schema)?;
                }
            } else {
                let new_ser_schema = self._walk_ser_schema(ser_schema)?;
                schema.set_item(serialization_key, new_ser_schema)?;
            }
        }
        Ok(schema.clone())
    }

    fn _handle_any_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_none_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_bool_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_int_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_float_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_decimal_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_string_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_bytes_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_date_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_time_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_datetime_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_timedelta_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_literal_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_is_instance_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_is_subclass_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_callable_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_list_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_items_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_tuple_positional_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let items_schema_key = intern2!(schema.py(), "items_schema");
        let items_schema = schema
            .get_item(items_schema_key)?
            .map(PyAnyMethods::downcast_into::<PyList>)
            .transpose()?;
        if let Some(items_schema) = items_schema {
            let new_items_schema = items_schema
                .iter()
                .map(|item_schema| self.walk(item_schema.downcast()?))
                .collect::<PyResult<Vec<Py2<'_, PyDict>>>>()?;
            schema.set_item(items_schema_key, new_items_schema)?;
        }
        let schema = &self._handle_extras_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_tuple_variable_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_items_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_set_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_items_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_frozenset_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_items_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_generator_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_items_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_dict_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let keys_schema: Option<Py2<'_, PyDict>> = schema
            .get_item("keys_schema")?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(keys_schema) = &keys_schema {
            let new_keys_schema = self.walk(keys_schema)?;
            schema.set_item("keys_schema", new_keys_schema)?;
        }
        let values_schema: Option<Py2<'_, PyDict>> = schema
            .get_item("values_schema")?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(values_schema) = &values_schema {
            let new_values_schema = self.walk(values_schema)?;
            schema.set_item("values_schema", new_values_schema)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_after_validator_function_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_before_validator_function_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_wrap_validator_function_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_plain_validator_function_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_with_default_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_nullable_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_union_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let choices_key = intern2!(schema.py(), "choices");
        let choices: Option<Py2<'_, PyList>> = schema
            .get_item(choices_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(choices) = choices {
            let py = schema.py();
            let new_choices = choices
                .iter()
                .map(
                    |choice| match choice.extract::<(Py2<'_, PyDict>, Py2<'_, PyString>)>() {
                        Ok(choice) => {
                            let (schema, tag) = &choice;
                            let schema = self.walk(schema)?;
                            Ok(PyTuple::new2(py, [schema.into_py(py), tag.into_py(py)]).into())
                        }
                        Err(_) => Ok(self.walk(choice.downcast()?)?.into()),
                    },
                )
                .collect::<PyResult<Vec<PyObject>>>()?;
            schema.set_item(choices_key, new_choices)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_tagged_union_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let choices_key = intern2!(schema.py(), "choices");
        let choices: Option<Py2<'_, PyDict>> = schema
            .get_item(choices_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(choices) = choices {
            let new_choices = choices.iter().map(|(k, v)| {
                let new_v = self.walk(v.downcast()?);
                Ok((k, new_v?))
            });
            schema.set_item(choices_key, py_dict_from_iterator(schema.py(), new_choices)?)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_chain_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let steps_key = intern2!(schema.py(), "steps");
        let steps: Option<Py2<'_, PyList>> = schema
            .get_item(steps_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(steps) = steps {
            let new_steps = steps
                .iter()
                .map(|step| self.walk(step.downcast()?))
                .collect::<PyResult<Vec<Py2<'_, PyDict>>>>()?;
            schema.set_item(steps_key, new_steps)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_lax_or_strict_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let lax_schema_key = intern2!(schema.py(), "lax_schema");
        let lax_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(lax_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(lax_schema) = &lax_schema {
            let new_lax_schema = self.walk(lax_schema)?;
            schema.set_item(lax_schema_key, new_lax_schema)?;
        }
        let strict_schema_key = intern2!(schema.py(), "strict_schema");
        let strict_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(strict_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(strict_schema) = &strict_schema {
            let new_strict_schema = self.walk(strict_schema)?;
            schema.set_item(strict_schema_key, new_strict_schema)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_json_or_python_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let json_schema_key = intern2!(schema.py(), "json_schema");
        let json_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(json_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(json_schema) = &json_schema {
            let new_json_schema = self.walk(json_schema)?;
            schema.set_item(json_schema_key, new_json_schema)?;
        }
        let python_schema_key = intern2!(schema.py(), "python_schema");
        let python_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(python_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(python_schema) = &python_schema {
            let new_python_schema = self.walk(python_schema)?;
            schema.set_item(python_schema_key, new_python_schema)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_typed_dict_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let fields_key = intern2!(schema.py(), "fields");
        let fields: Option<Py2<'_, PyDict>> = schema
            .get_item(fields_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(fields) = fields {
            let new_fields = fields.iter().map(|(k, v)| {
                let typed_dict_field = v.downcast_into::<PyDict>()?;
                let schema = &typed_dict_field
                    .get_item("schema")
                    .ok()
                    .flatten()
                    .ok_or_else(|| PyTypeError::new_err("Missing schema in TypedDictField"))?
                    .downcast_into()?;
                let new_schema = self.walk(schema)?;
                typed_dict_field.set_item("schema", new_schema)?;
                Ok((k, typed_dict_field))
            });
            schema.set_item(fields_key, py_dict_from_iterator(schema.py(), new_fields)?)?;
        };
        let schema = &self._handle_extras_schema(schema)?;
        let schema = &self._handle_computed_fields_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_model_field_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?;
        self._check_ser_schema(schema)
    }

    fn _handle_model_fields_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let fields_key = intern2!(schema.py(), "fields");
        let fields: Option<Py2<'_, PyDict>> = schema
            .get_item(fields_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(fields) = fields {
            let new_fields = fields.iter().map(|(k, v)| {
                let new_v = self.walk(v.downcast()?)?;
                Ok((k, new_v))
            });
            schema.set_item(fields_key, py_dict_from_iterator(schema.py(), new_fields)?)?;
        }
        let schema = &self._handle_extras_schema(schema)?;
        let schema = &self._handle_computed_fields_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_model_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?;
        self._check_ser_schema(schema)
    }

    fn _handle_dataclass_args_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let fields_key = intern2!(schema.py(), "fields");
        let schema_key = intern2!(schema.py(), "schema");
        let fields: Option<Py2<'_, PyList>> = schema
            .get_item(fields_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(fields) = fields {
            for v in fields {
                let dataclass_field = v.downcast_into::<PyDict>()?;
                let dataclass_field_schema = &dataclass_field.get_item(schema_key)?.unwrap().downcast_into()?;
                let new_dataclass_field_schema = self.walk(dataclass_field_schema)?;
                dataclass_field.set_item(schema_key, new_dataclass_field_schema)?;
            }
        }
        let schema = &self._handle_computed_fields_schema(schema)?;
        self._check_ser_schema(schema)
    }

    fn _handle_dataclass_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_arguments_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let arguments_schema_key = intern2!(schema.py(), "arguments_schema");
        let arguments_schema: Option<Py2<'_, PyList>> = schema
            .get_item(arguments_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(arguments_schema) = arguments_schema {
            for argument_parameter in arguments_schema {
                let argument_parameter = argument_parameter.downcast_into::<PyDict>()?;
                let argument_schema = &argument_parameter
                    .get_item("schema")
                    .ok()
                    .flatten()
                    .ok_or_else(|| PyTypeError::new_err("Missing schema in ArgumentParameter"))?
                    .downcast_into()?;
                let new_argument_schema = self.walk(argument_schema)?;
                argument_parameter.set_item("schema", new_argument_schema)?;
            }
        }
        self._check_ser_schema(schema)
    }

    fn _handle_call_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let arguments_schema_key = intern2!(schema.py(), "arguments_schema");
        let return_schema_key = intern2!(schema.py(), "return_schema");
        let arguments_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(arguments_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(arguments_schema) = &arguments_schema {
            let new_arguments_schema = self.walk(arguments_schema)?;
            schema.set_item(arguments_schema_key, new_arguments_schema)?;
        }
        let return_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(return_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(return_schema) = &return_schema {
            let new_return_schema = self.walk(return_schema)?;
            schema.set_item(return_schema_key, new_return_schema)?;
        }
        self._check_ser_schema(schema)
    }

    fn _handle_custom_error_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_json_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_url_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_multi_host_url_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_definitions_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let definitions_key = intern2!(schema.py(), "definitions");
        let definitions: Option<Py2<'_, PyList>> = schema
            .get_item(definitions_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(definitions) = definitions {
            let new_definitions = definitions
                .iter()
                .map(|definition| self.walk(&definition.extract()?))
                .collect::<PyResult<Vec<Py2<'_, PyDict>>>>()?;
            schema.set_item(definitions_key, new_definitions)?;
        }
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._check_ser_schema(schema)
    }

    fn _handle_definition_reference_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_uuid_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._check_ser_schema(schema)
    }

    fn _handle_simple_ser_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        Ok(schema.clone())
    }

    fn _handle_plain_serializer_function_ser_schema<'py>(
        &self,
        schema: &Py2<'py, PyDict>,
    ) -> PyResult<Py2<'py, PyDict>> {
        self._handle_inner_schema(schema, intern2!(schema.py(), "return_schema"))
    }

    fn _handle_wrap_serializer_function_ser_schema<'py>(
        &self,
        schema: &Py2<'py, PyDict>,
    ) -> PyResult<Py2<'py, PyDict>> {
        let schema = &self
            ._handle_inner_schema(schema, intern2!(schema.py(), "schema"))?
            .extract()?;
        self._handle_inner_schema(schema, intern2!(schema.py(), "return_schema"))
    }

    fn _handle_format_ser_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        Ok(schema.clone())
    }

    fn _handle_to_string_ser_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        Ok(schema.clone())
    }

    fn _handle_model_ser_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        self._handle_inner_schema(schema, intern2!(schema.py(), "schema"))
    }

    // Handle a dict where there may be a `"schema": CoreSchema` key
    fn _handle_inner_schema<'py>(
        &self,
        schema: &Py2<'py, PyDict>,
        schema_key: &Py2<'_, PyString>,
    ) -> PyResult<Py2<'py, PyDict>> {
        let inner_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(inner_schema) = &inner_schema {
            let new_inner_schema = self.walk(inner_schema)?;
            schema.set_item(schema_key, new_inner_schema)?;
        }
        Ok(schema.clone())
    }

    // Handle a dict where there may be a `"items_schema": CoreSchema` key
    fn _handle_items_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let items_schema_key = intern2!(schema.py(), "items_schema");
        let items_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(items_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(items_schema) = &items_schema {
            let new_items_schema = self.walk(items_schema)?;
            schema.set_item(items_schema_key, new_items_schema)?;
        }
        Ok(schema.clone())
    }

    fn _handle_computed_fields_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let computed_fields_key = intern2!(schema.py(), "computed_fields");
        let computed_fields: Option<Py2<'_, PyList>> = schema
            .get_item(computed_fields_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(computed_fields) = computed_fields {
            let schema_key = intern2!(schema.py(), "schema");
            let return_schema_key = intern2!(schema.py(), "return_schema");
            let new_computed_fields = computed_fields
                .iter()
                .map(|computed_field| {
                    let computed_field_schema = &computed_field.extract()?;
                    let computed_field_schema = &self._handle_inner_schema(computed_field_schema, schema_key)?;
                    let computed_field_schema = self._handle_inner_schema(computed_field_schema, return_schema_key)?;
                    Ok(computed_field_schema)
                })
                .collect::<PyResult<Vec<Py2<'_, PyDict>>>>()?;
            schema.set_item(computed_fields_key, new_computed_fields)?;
        }
        Ok(schema.clone())
    }

    fn _handle_extras_schema<'py>(&self, schema: &Py2<'py, PyDict>) -> PyResult<Py2<'py, PyDict>> {
        let extras_schema_key = intern2!(schema.py(), "extras_schema");
        let extras_schema: Option<Py2<'_, PyDict>> = schema
            .get_item(extras_schema_key)?
            .map(PyAnyMethods::downcast_into)
            .transpose()?;
        if let Some(extras_schema) = &extras_schema {
            let new_extras_schema = self.walk(extras_schema)?;
            schema.set_item(extras_schema_key, new_extras_schema)?;
        }
        Ok(schema.clone())
    }
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
    fn matches(&self, schema: &Py2<'_, PyDict>) -> PyResult<bool> {
        match self {
            Filter::HasRef => {
                let ref_ = schema.get_item("ref")?;
                Ok(ref_.is_some())
            }
            Filter::HasType { type_ } => {
                if let Some(schema_type) = schema
                    .get_item("type")?
                    .map(PyAnyMethods::downcast_into::<PyString>)
                    .transpose()?
                {
                    Ok(schema_type.to_str()? == type_)
                } else {
                    Ok(false)
                }
            }
            Filter::Python { predicate } => {
                let result: bool = predicate.attach(schema.py()).call1((schema,))?.extract()?;
                Ok(result)
            }
            Filter::And { left, right } => Ok(left.matches(schema)? && right.matches(schema)?),
            Filter::Or { left, right } => Ok(left.matches(schema)? || right.matches(schema)?),
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
    fn matches(&self, schema: &Py2<'_, PyDict>) -> PyResult<bool> {
        self.filter.matches(schema)
    }

    fn call<'py>(&self, schema: &Py2<'py, PyDict>, call_next: PyObject) -> PyResult<Py2<'py, PyDict>> {
        self.func
            .attach(schema.py())
            .call1((schema, call_next))?
            .downcast_into()
            .map_err(Into::into)
    }
}

fn py_dict_from_iterator<K: ToPyObject, V: ToPyObject>(
    py: Python,
    iterator: impl IntoIterator<Item = PyResult<(K, V)>>,
) -> PyResult<Py2<'_, PyDict>> {
    let dict = PyDict::new2(py);
    for item in iterator {
        let (k, v) = item?;
        dict.set_item(k, v)?;
    }
    Ok(dict)
}
