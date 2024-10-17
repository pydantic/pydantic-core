use crate::tools::py_err;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyString, PyTuple};
use pyo3::{intern, Bound, PyResult};
use std::collections::HashSet;

const CORE_SCHEMA_METADATA_DISCRIMINATOR_PLACEHOLDER_KEY: &str = "pydantic.internal.union_discriminator";

macro_rules! get {
    ($dict: expr, $key: expr) => {{
        $dict.get_item(intern!($dict.py(), $key))?
    }};
}

macro_rules! traverse {
    ($func: expr, $key: expr, $dict: expr, $ctx: expr) => {{
        if let Some(v) = $dict.get_item(intern!($dict.py(), $key))? {
            $func(v.downcast_exact()?, $ctx)?
        }
    }};
}

macro_rules! defaultdict_list_append {
    ($dict: expr, $key: expr, $value: expr) => {{
        match $dict.get_item($key)? {
            None => {
                let list = PyList::empty_bound($dict.py());
                list.append($value)?;
                $dict.set_item($key, list)?;
            }
            // Safety: we know that the value is a PyList as we just created it above
            Some(list) => unsafe { list.downcast_unchecked::<PyList>() }.append($value)?,
        };
    }};
}

fn gather_definition_ref(schema_ref_dict: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<bool> {
    if let Some(schema_ref) = get!(schema_ref_dict, "schema_ref") {
        let schema_ref_pystr = schema_ref.downcast_exact::<PyString>()?;
        let schema_ref_str = schema_ref_pystr.to_str()?;

        if !ctx.seen_refs.contains(schema_ref_str) {
            defaultdict_list_append!(&ctx.def_refs, schema_ref_pystr, schema_ref_dict);

            if let Some(def) = ctx.definitions_dict.get_item(schema_ref_pystr)? {
                ctx.seen_refs.insert(schema_ref_str.to_string());
                gather_schema(def.downcast_exact::<PyDict>()?, ctx)?;
                ctx.seen_refs.remove(schema_ref_str);
            }
            Ok(false)
        } else {
            ctx.recursive_def_refs.add(schema_ref_pystr)?;
            Ok(true)
        }
    } else {
        py_err!(PyKeyError; "Invalid definition-ref, missing schema_ref")?
    }
}

fn gather_meta(schema: &Bound<'_, PyDict>, meta_dict: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    if let Some(discriminator) = get!(meta_dict, CORE_SCHEMA_METADATA_DISCRIMINATOR_PLACEHOLDER_KEY) {
        let schema_discriminator = PyTuple::new_bound(schema.py(), vec![schema.as_any(), &discriminator]);
        ctx.discriminators.append(schema_discriminator)?;
    }
    Ok(())
}

fn gather_node(schema: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    let type_ = get!(schema, "type");
    if type_.is_none() {
        return py_err!(PyValueError; "Schema type missing");
    }
    if type_.unwrap().downcast_exact::<PyString>()?.to_str()? == "definition-ref" {
        let recursive = gather_definition_ref(schema, ctx)?;
        if recursive {
            return Ok(());
        }
    }
    if let Some(meta) = get!(schema, "metadata") {
        gather_meta(schema, meta.downcast_exact()?, ctx)?;
    }
    Ok(())
}

fn gather_list(schema_list: &Bound<'_, PyList>, ctx: &mut GatherCtx) -> PyResult<()> {
    for v in schema_list.iter() {
        gather_schema(v.downcast_exact()?, ctx)?;
    }
    Ok(())
}

fn gather_dict(schemas_by_key: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    for (_, v) in schemas_by_key.iter() {
        gather_schema(v.downcast_exact()?, ctx)?;
    }
    Ok(())
}

fn gather_union_choices(schema_list: &Bound<'_, PyList>, ctx: &mut GatherCtx) -> PyResult<()> {
    for v in schema_list.iter() {
        if let Ok(tup) = v.downcast_exact::<PyTuple>() {
            gather_schema(tup.get_item(0)?.downcast_exact()?, ctx)?;
        } else {
            gather_schema(v.downcast_exact()?, ctx)?;
        }
    }
    Ok(())
}

fn gather_arguments(arguments: &Bound<'_, PyList>, ctx: &mut GatherCtx) -> PyResult<()> {
    for v in arguments.iter() {
        if let Some(schema) = get!(v.downcast_exact::<PyDict>()?, "schema") {
            gather_schema(schema.downcast_exact()?, ctx)?;
        }
    }
    Ok(())
}

fn traverse_schema(schema: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    let type_ = get!(schema, "type");
    if type_.is_none() {
        return py_err!(PyValueError; "Schema type missing");
    }
    match type_.unwrap().downcast_exact::<PyString>()?.to_str()? {
        "definitions" => {
            traverse!(gather_schema, "schema", schema, ctx);
            traverse!(gather_list, "definitions", schema, ctx);
        }
        "list" | "set" | "frozenset" | "generator" => traverse!(gather_schema, "items_schema", schema, ctx),
        "tuple" => traverse!(gather_list, "items_schema", schema, ctx),
        "dict" => {
            traverse!(gather_schema, "keys_schema", schema, ctx);
            traverse!(gather_schema, "values_schema", schema, ctx);
        }
        "union" => traverse!(gather_union_choices, "choices", schema, ctx),
        "tagged-union" => traverse!(gather_dict, "choices", schema, ctx),
        "chain" => traverse!(gather_list, "steps", schema, ctx),
        "lax-or-strict" => {
            traverse!(gather_schema, "lax_schema", schema, ctx);
            traverse!(gather_schema, "strict_schema", schema, ctx);
        }
        "json-or-python" => {
            traverse!(gather_schema, "json_schema", schema, ctx);
            traverse!(gather_schema, "python_schema", schema, ctx);
        }
        "model-fields" | "typed-dict" => {
            traverse!(gather_schema, "extras_schema", schema, ctx);
            traverse!(gather_list, "computed_fields", schema, ctx);
            traverse!(gather_dict, "fields", schema, ctx);
        }
        "dataclass-args" => {
            traverse!(gather_list, "computed_fields", schema, ctx);
            traverse!(gather_list, "fields", schema, ctx);
        }
        "arguments" => {
            traverse!(gather_arguments, "arguments_schema", schema, ctx);
            traverse!(gather_schema, "var_args_schema", schema, ctx);
            traverse!(gather_schema, "var_kwargs_schema", schema, ctx);
        }
        "computed-field" | "function-plain" => traverse!(gather_schema, "return_schema", schema, ctx),
        "function-wrap" => {
            traverse!(gather_schema, "return_schema", schema, ctx);
            traverse!(gather_schema, "schema", schema, ctx);
        }
        "call" => {
            traverse!(gather_schema, "arguments_schema", schema, ctx);
            traverse!(gather_schema, "return_schema", schema, ctx);
        }
        _ => traverse!(gather_schema, "schema", schema, ctx),
    };

    if let Some(ser) = get!(schema, "serialization") {
        let ser_dict = ser.downcast_exact::<PyDict>()?;
        traverse!(gather_schema, "schema", ser_dict, ctx);
        traverse!(gather_schema, "return_schema", ser_dict, ctx);
    }
    Ok(())
}

pub struct GatherCtx<'a, 'py> {
    pub definitions_dict: &'a Bound<'py, PyDict>,
    pub def_refs: Bound<'py, PyDict>,
    pub recursive_def_refs: Bound<'py, PySet>,
    pub discriminators: Bound<'py, PyList>,
    seen_refs: HashSet<String>,
}

impl<'a, 'py> GatherCtx<'a, 'py> {
    pub fn new(definitions: &'a Bound<'py, PyDict>) -> PyResult<Self> {
        let ctx = GatherCtx {
            definitions_dict: definitions,
            def_refs: PyDict::new_bound(definitions.py()),
            recursive_def_refs: PySet::empty_bound(definitions.py())?,
            discriminators: PyList::empty_bound(definitions.py()),
            seen_refs: HashSet::new(),
        };
        Ok(ctx)
    }
}

pub fn gather_schema(schema: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    traverse_schema(schema, ctx)?;
    gather_node(schema, ctx)
}
