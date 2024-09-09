use std::ops::{Add, Div, Mul, Sub};

use crate::error::ValueError;

#[derive(Clone)]
pub enum Value<'a> {
    Owned(OwnedValue),
    Borrowed(&'a OwnedValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OwnedValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<OwnedValue>),
}

impl<'a> Value<'a> {
    pub fn inner(&self) -> &OwnedValue {
        match self {
            Value::Owned(val) => val,
            Value::Borrowed(val) => *val,
        }
    }

    pub fn unwrap_string(self) -> String {
        match self {
            Value::Owned(val) => match val {
                OwnedValue::String(string) => string,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn unwrap_f64(self) -> Result<f64, ValueError> {
        match self.inner() {
            OwnedValue::Number(num) => Ok(*num),
            val => Err(ValueError::OperationError(format!(
                "Cannot unwrap {val:?} as f64"
            ))),
        }
    }

    pub fn to_owned_value(&self) -> OwnedValue {
        match self {
            Value::Borrowed(value) => (*value).to_owned(),
            Value::Owned(value) => value.to_owned(),
        }
    }
}

impl OwnedValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            OwnedValue::String(string) => !string.is_empty(),
            OwnedValue::Number(number) => *number != 0.0,
            OwnedValue::Boolean(boolean) => *boolean,
            OwnedValue::Array(vec) => !vec.is_empty(),
        }
    }
}

impl ToString for OwnedValue {
    fn to_string(&self) -> String {
        match self {
            OwnedValue::String(string) => string.to_owned(),
            OwnedValue::Number(num) => num.to_string(),
            OwnedValue::Boolean(boolean) => boolean.to_string(),
            OwnedValue::Array(vec) => vec
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

impl Add<&OwnedValue> for &OwnedValue {
    type Output = Result<OwnedValue, ValueError>;

    fn add(self, rhs: &OwnedValue) -> Self::Output {
        match (self, rhs) {
            (OwnedValue::Number(lhs), OwnedValue::Number(rhs)) => Ok(OwnedValue::Number(lhs + rhs)),
            (OwnedValue::Number(lhs), OwnedValue::String(rhs)) => {
                Ok(OwnedValue::String(format!("{lhs}{rhs}")))
            }
            (OwnedValue::String(lhs), OwnedValue::Number(rhs)) => {
                Ok(OwnedValue::String(format!("{lhs}{rhs}")))
            }
            (OwnedValue::String(lhs), OwnedValue::String(rhs)) => {
                Ok(OwnedValue::String(format!("{lhs}{rhs}")))
            }
            (OwnedValue::Array(lhs), OwnedValue::Array(rhs)) => Ok(OwnedValue::Array(
                lhs.iter().chain(rhs).cloned().collect::<Vec<OwnedValue>>(),
            )),
            _ => Err(ValueError::OperationError(format!(
                "Cannot add {rhs:?} to {self:?}"
            ))),
        }
    }
}

impl Sub<&OwnedValue> for &OwnedValue {
    type Output = Result<OwnedValue, ValueError>;

    fn sub(self, rhs: &OwnedValue) -> Self::Output {
        match (self, rhs) {
            (OwnedValue::Number(lhs), OwnedValue::Number(rhs)) => Ok(OwnedValue::Number(lhs - rhs)),
            _ => Err(ValueError::OperationError(format!(
                "Cannot subtract {rhs:?} from {self:?}"
            ))),
        }
    }
}

impl Mul<&OwnedValue> for &OwnedValue {
    type Output = Result<OwnedValue, ValueError>;

    fn mul(self, rhs: &OwnedValue) -> Self::Output {
        match (self, rhs) {
            (OwnedValue::Number(lhs), OwnedValue::Number(rhs)) => Ok(OwnedValue::Number(lhs * rhs)),
            (OwnedValue::String(lhs), OwnedValue::Number(rhs)) => {
                Ok(OwnedValue::String(lhs.repeat(*rhs as usize)))
            }
            (OwnedValue::Number(lhs), OwnedValue::String(rhs)) => {
                Ok(OwnedValue::String(rhs.repeat(*lhs as usize)))
            }
            _ => Err(ValueError::OperationError(format!(
                "Cannot multiply strings {self:?} with {rhs:?}"
            ))),
        }
    }
}

impl Div<&OwnedValue> for &OwnedValue {
    type Output = Result<OwnedValue, ValueError>;

    fn div(self, rhs: &OwnedValue) -> Self::Output {
        match (self, rhs) {
            (OwnedValue::Number(lhs), OwnedValue::Number(rhs)) => Ok(OwnedValue::Number(lhs / rhs)),
            _ => Err(ValueError::OperationError(format!(
                "Cannot divide {self:?} by {rhs:?}"
            ))),
        }
    }
}

impl Into<Value<'_>> for OwnedValue {
    fn into(self) -> Value<'static> {
        Value::Owned(self)
    }
}

impl<'a> Into<Value<'a>> for &'a OwnedValue {
    fn into(self) -> Value<'a> {
        Value::Borrowed(self)
    }
}

impl Into<Value<'_>> for String {
    fn into(self) -> Value<'static> {
        Value::Owned(OwnedValue::String(self))
    }
}

impl Into<Value<'_>> for bool {
    fn into(self) -> Value<'static> {
        Value::Owned(OwnedValue::Boolean(self))
    }
}

impl Into<Value<'_>> for f64 {
    fn into(self) -> Value<'static> {
        Value::Owned(OwnedValue::Number(self))
    }
}

impl Into<Value<'_>> for Vec<OwnedValue> {
    fn into(self) -> Value<'static> {
        Value::Owned(OwnedValue::Array(self))
    }
}
