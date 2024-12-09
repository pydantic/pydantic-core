use crate::tools::py_err;
use pyo3::exceptions::{PyException, PyKeyError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyNone, PySet, PyString, PyTuple};
use pyo3::{create_exception, intern, Bound, PyResult};

create_exception!(pydantic_core._pydantic_core, GatherInvalidDefinitionError, PyException);

macro_rules! none {
    ($py: expr) => {
        PyNone::get_bound($py)
    };
}

macro_rules! get {
    ($dict: expr, $key: expr) => {
        $dict.get_item(intern!($dict.py(), $key))?
    };
}

macro_rules! traverse_key_fn {
    ($key: expr, $func: expr, $dict: expr, $ctx: expr) => {{
        if let Some(v) = get!($dict, $key) {
            $func(v.downcast_exact()?, $ctx)?
        }
    }};
}

macro_rules! traverse {
    ($($key:expr => $func:expr),*; $dict: expr, $ctx: expr) => {{
        $(traverse_key_fn!($key, $func, $dict, $ctx);)*
        traverse_key_fn!("serialization", gather_schema, $dict, $ctx);
        gather_meta($dict, $ctx)
    }}
}

macro_rules! defaultdict_list_append {
    ($dict: expr, $key: expr, $value: expr) => {{
        match $dict.get_item($key)? {
            None => {
                let list = PyList::new_bound($dict.py(), [$value]);
                $dict.set_item($key, list)?;
            }
            // Safety: we know that the value is a PyList as we just created it above
            Some(list) => unsafe { list.downcast_unchecked::<PyList>() }.append($value)?,
        };
    }};
}

fn gather_definition_ref(schema_ref_dict: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    let Some(schema_ref) = get!(schema_ref_dict, "schema_ref") else {
        return py_err!(PyKeyError; "Invalid definition-ref, missing schema_ref");
    };
    let schema_ref = schema_ref.downcast_exact::<PyString>()?;
    let py = schema_ref_dict.py();

    if !ctx.recursively_seen_refs.contains(schema_ref)? {
        // Def ref in no longer consider as inlinable if its re-encountered. Then its used multiple times.
        // No need to retraverse it either if we already encountered this.
        if !ctx.inline_def_ref_candidates.contains(schema_ref)? {
            let Some(definition) = ctx.definitions.get_item(schema_ref)? else {
                return py_err!(GatherInvalidDefinitionError; "{}", schema_ref.to_str()?);
            };

            ctx.inline_def_ref_candidates.set_item(schema_ref, schema_ref_dict)?;
            ctx.recursively_seen_refs.add(schema_ref)?;

            gather_schema(definition.downcast_exact()?, ctx)?;
            traverse_key_fn!("serialization", gather_schema, schema_ref_dict, ctx);
            gather_meta(schema_ref_dict, ctx)?;

            ctx.recursively_seen_refs.discard(schema_ref)?;
        } else {
            ctx.inline_def_ref_candidates.set_item(schema_ref, none!(py))?; // Mark not inlinable (used multiple times)
        }
    } else {
        ctx.inline_def_ref_candidates.set_item(schema_ref, none!(py))?; // Mark not inlinable (used in recursion)
        ctx.recursive_def_refs.add(schema_ref)?;
        for seen_ref in ctx.recursively_seen_refs.iter() {
            ctx.inline_def_ref_candidates.set_item(&seen_ref, none!(py))?; // Mark not inlinable (used in recursion)
            ctx.recursive_def_refs.add(seen_ref)?;
        }
    }
    Ok(())
}

fn gather_meta(schema: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    let Some((res, find_keys)) = &ctx.meta_with_keys else {
        return Ok(());
    };
    let Some(meta) = get!(schema, "metadata") else {
        return Ok(());
    };
    let meta_dict = meta.downcast_exact::<PyDict>()?;
    for k in find_keys.iter() {
        if meta_dict.contains(&k)? {
            defaultdict_list_append!(res, &k, schema);
        }
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
        traverse_key_fn!("schema", gather_schema, v.downcast_exact::<PyDict>()?, ctx);
    }
    Ok(())
}

// Has 100% coverage in Pydantic side. This is exclusively used there
#[cfg_attr(has_coverage_attribute, coverage(off))]
fn gather_schema(schema: &Bound<'_, PyDict>, ctx: &mut GatherCtx) -> PyResult<()> {
    let Some(type_) = get!(schema, "type") else {
        return py_err!(PyKeyError; "Schema type missing");
    };
    match type_.downcast_exact::<PyString>()?.to_str()? {
        "definition-ref" => gather_definition_ref(schema, ctx),
        "definitions" => traverse!("schema" => gather_schema, "definitions" => gather_list; schema, ctx),
        "list" | "set" | "frozenset" | "generator" => traverse!("items_schema" => gather_schema; schema, ctx),
        "tuple" => traverse!("items_schema" => gather_list; schema, ctx),
        "dict" => traverse!("keys_schema" => gather_schema, "values_schema" => gather_schema; schema, ctx),
        "union" => traverse!("choices" => gather_union_choices; schema, ctx),
        "tagged-union" => traverse!("choices" => gather_dict; schema, ctx),
        "chain" => traverse!("steps" => gather_list; schema, ctx),
        "lax-or-strict" => traverse!("lax_schema" => gather_schema, "strict_schema" => gather_schema; schema, ctx),
        "json-or-python" => traverse!("json_schema" => gather_schema, "python_schema" => gather_schema; schema, ctx),
        "model-fields" | "typed-dict" => traverse!(
            "extras_schema" => gather_schema, "computed_fields" => gather_list, "fields" => gather_dict; schema, ctx
        ),
        "dataclass-args" => traverse!("computed_fields" => gather_list, "fields" => gather_list; schema, ctx),
        "arguments" => traverse!(
            "arguments_schema" => gather_arguments,
            "var_args_schema" => gather_schema,
            "var_kwargs_schema" => gather_schema;
            schema, ctx
        ),
        "call" => traverse!("arguments_schema" => gather_schema, "return_schema" => gather_schema; schema, ctx),
        "computed-field" | "function-plain" => traverse!("return_schema" => gather_schema; schema, ctx),
        "function-wrap" => traverse!("return_schema" => gather_schema, "schema" => gather_schema; schema, ctx),
        _ => traverse!("schema" => gather_schema; schema, ctx),
    }
}

struct GatherCtx<'a, 'py> {
    definitions: &'a Bound<'py, PyDict>,
    meta_with_keys: Option<(Bound<'py, PyDict>, &'a Bound<'py, PySet>)>,
    inline_def_ref_candidates: Bound<'py, PyDict>,
    recursive_def_refs: Bound<'py, PySet>,
    recursively_seen_refs: Bound<'py, PySet>,
}

#[pyfunction(signature = (schema, definitions, find_meta_with_keys))]
pub fn gather_schemas_for_cleaning<'py>(
    schema: &Bound<'py, PyAny>,
    definitions: &Bound<'py, PyAny>,
    find_meta_with_keys: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyDict>> {
    let py = schema.py();
    let mut ctx = GatherCtx {
        definitions: definitions.downcast_exact()?,
        meta_with_keys: match find_meta_with_keys.is_none() {
            true => None,
            false => Some((PyDict::new_bound(py), find_meta_with_keys.downcast_exact::<PySet>()?)),
        },
        inline_def_ref_candidates: PyDict::new_bound(py),
        recursive_def_refs: PySet::empty_bound(py)?,
        recursively_seen_refs: PySet::empty_bound(py)?,
    };
    gather_schema(schema.downcast_exact()?, &mut ctx)?;

    let res = PyDict::new_bound(py);
    res.set_item(intern!(py, "inlinable_def_refs"), ctx.inline_def_ref_candidates)?;
    res.set_item(intern!(py, "recursive_refs"), ctx.recursive_def_refs)?;
    res.set_item(intern!(py, "schemas_with_meta_keys"), ctx.meta_with_keys.map(|v| v.0))?;
    Ok(res)
}
