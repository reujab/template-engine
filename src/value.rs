use std::ops::{Add, Div, Mul, Sub};

use crate::error::ValueError;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(string) => string.to_owned(),
            Value::Number(num) => num.to_string(),
        }
    }
}

impl Add<&Value> for &Value {
    type Output = Value;

    fn add(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
            (Value::Number(lhs), Value::String(rhs)) => Value::String(format!("{lhs}{rhs}")),
            (Value::String(lhs), Value::Number(rhs)) => Value::String(format!("{lhs}{rhs}")),
            (Value::String(lhs), Value::String(rhs)) => Value::String(format!("{lhs}{rhs}")),
        }
    }
}

impl Sub<&Value> for &Value {
    type Output = Result<Value, ValueError>;

    fn sub(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),
            _ => Err(ValueError::StringOpError(format!(
                "Cannot subtract {rhs:?} from {self:?}"
            ))),
        }
    }
}

impl Mul<&Value> for &Value {
    type Output = Result<Value, ValueError>;

    fn mul(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),
            (Value::String(lhs), Value::Number(rhs)) => {
                Ok(Value::String(lhs.repeat(*rhs as usize)))
            }
            (Value::Number(lhs), Value::String(rhs)) => {
                Ok(Value::String(rhs.repeat(*lhs as usize)))
            }
            _ => Err(ValueError::StringOpError(format!(
                "Cannot multiply strings {self:?} with {rhs:?}"
            ))),
        }
    }
}

impl Div<&Value> for &Value {
    type Output = Result<Value, ValueError>;

    fn div(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),
            _ => Err(ValueError::StringOpError(format!(
                "Cannot divide {self:?} by {rhs:?}"
            ))),
        }
    }
}
