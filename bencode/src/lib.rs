mod borrow;
mod error;
mod owned;

pub use crate::borrow::Value as BorrowValue;
pub use crate::error::{Error, Result};
pub use crate::owned::Value;

impl BorrowValue<'_> {
    pub fn to_owned(&self) -> Value {
        use BorrowValue::*;

        match self {
            Int(n) => Value::Int(*n),
            String(buf) => Value::String(buf.to_vec()),
            List(list) => Value::List(list.iter().map(|v| v.to_owned()).collect()),
            Dict(dict) => Value::Dict(
                dict.iter()
                    .map(|(&k, v)| (k.to_owned(), v.to_owned()))
                    .collect(),
            ),
        }
    }
}

impl Value {
    pub fn to_borrow(&self) -> BorrowValue {
        use Value::*;
        match self {
            Int(n) => BorrowValue::Int(*n),
            String(buf) => BorrowValue::String(&buf),
            List(list) => BorrowValue::List(list.iter().map(|v| v.to_borrow()).collect()),
            Dict(dict) => BorrowValue::Dict(
                dict.iter()
                    .map(|(k, v)| (k.as_ref(), v.to_borrow()))
                    .collect(),
            ),
        }
    }
}
