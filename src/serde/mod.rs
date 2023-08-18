// Ignored clippy lints
#![allow(
    clippy::collapsible_else_if,
    clippy::comparison_chain,
    clippy::deprecated_cfg_attr,
    clippy::doc_markdown,
    clippy::excessive_precision,
    clippy::explicit_auto_deref,
    clippy::float_cmp,
    clippy::manual_range_contains,
    clippy::match_like_matches_macro,
    clippy::match_single_binding,
    clippy::needless_doctest_main,
    clippy::needless_late_init,
    clippy::return_self_not_must_use,
    clippy::transmute_ptr_to_ptr,
    clippy::unnecessary_wraps
)]
// Ignored clippy_pedantic lints
#![allow(
    // Deserializer::from_str, into_iter
    clippy::should_implement_trait,
    // integer and float ser/de requires these sorts of casts
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    // correctly used
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::integer_division,
    clippy::let_underscore_untyped,
    clippy::map_err_ignore,
    clippy::match_same_arms,
    clippy::similar_names,
    clippy::unused_self,
    clippy::wildcard_imports,
    // things are often more readable this way
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::redundant_else,
    clippy::shadow_unrelated,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::use_self,
    clippy::zero_prefixed_literal,
    // we support older compilers
    clippy::checked_conversions,
    clippy::mem_replace_with_default,
    // noisy
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unnested_or_patterns,
)]
// Restrictions
#![allow(non_upper_case_globals)]
#![deny(missing_docs)]

mod de;
mod error;
mod read;
mod ser;
mod shared;

pub(crate) use de::{from_slice, from_str};
pub(crate) use error::PydanticSerdeError;
pub(crate) use ser::PythonSerializer;
