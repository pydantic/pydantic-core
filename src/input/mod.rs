use pyo3::prelude::*;

mod datetime;
mod input_abstract;
mod input_json;
mod input_python;
mod parse_json;
mod return_enums;
mod shared;

pub use datetime::{EitherDate, EitherDateTime, EitherTime, EitherTimedelta};
pub use input_abstract::Input;
pub use parse_json::{json_object_to_py, JsonInput, JsonObject};
pub use return_enums::{
    EitherBytes, EitherString, GenericArguments, GenericListLike, GenericMapping, JsonArgs, PyArgs,
};

pub fn repr_string(v: &PyAny) -> PyResult<String> {
    v.repr()?.extract()
}
