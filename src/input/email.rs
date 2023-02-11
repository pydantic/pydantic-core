use std::borrow::Cow;

use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use mail_parser::parsers::MessageStream;
use mail_parser::{Addr, HeaderValue};

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum NameEmail<'a> {
    Raw(Addr<'a>),
}

impl<'a> From<Addr<'a>> for NameEmail<'a> {
    fn from(date: Addr<'a>) -> Self {
        Self::Raw(date)
    }
}

// TODO: actually handle lifetimes
pub fn bytes_as_email_name<'a>(input: &'a impl Input<'a>, bytes: &'a [u8]) -> ValResult<'a, NameEmail<'a>> {
    match MessageStream::new(bytes).parse_address() {
        // TODO: FIX
        HeaderValue::Address(addr) => Ok(addr.into()),
        _ => Err(ValError::new(
            // TODO: FIX
            ErrorType::DateParsing {
                error: Cow::Borrowed("1234"),
            },
            input,
        )),
    }
}
