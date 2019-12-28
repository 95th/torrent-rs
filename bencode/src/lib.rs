macro_rules! impl_is_ty {
    ($($fn: ident == $type: ident,)*) => {
        $(
            pub fn $fn(&self) -> bool {
                if let Self::$type(_) = self {
                    true
                } else {
                    false
                }
            }
        )*
    }
}

macro_rules! inner_if {
    ($self: ident == $ty: ident) => {
        match $self {
            Self::$ty(inner) => Some(inner),
            _ => None,
        }
    };
}

mod error;
mod reader;
mod value;
mod value_ref;

pub use crate::error::{Error, Result};
pub use crate::value::Value;
pub use crate::value_ref::ValueRef;

impl ValueRef<'_> {
    pub fn to_owned(&self) -> Value {
        match self {
            ValueRef::Int(n) => Value::Int(*n),
            ValueRef::Bytes(buf) => Value::Bytes(buf.to_vec()),
            ValueRef::List(list) => Value::List(list.iter().map(|v| v.to_owned()).collect()),
            ValueRef::Dict(dict) => Value::Dict(
                dict.iter()
                    .map(|(&k, v)| (k.to_owned(), v.to_owned()))
                    .collect(),
            ),
        }
    }
}

impl Value {
    pub fn as_ref(&self) -> ValueRef {
        match self {
            Value::Int(n) => ValueRef::Int(*n),
            Value::Bytes(buf) => ValueRef::Bytes(&buf),
            Value::List(list) => ValueRef::List(list.iter().map(|v| v.as_ref()).collect()),
            Value::Dict(dict) => {
                ValueRef::Dict(dict.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect())
            }
        }
    }
}
