use std::ops::{Add, Div, Mul, Sub};

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
    type Output = Value;

    fn sub(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
            _ => panic!("Cannot subtract {rhs:?} from {self:?}"),
        }
    }
}

impl Mul<&Value> for &Value {
    type Output = Value;

    fn mul(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
            (Value::String(lhs), Value::Number(rhs)) => Value::String(lhs.repeat(*rhs as usize)),
            _ => panic!("Cannot multiple {rhs:?} with {self:?}"),
        }
    }
}

impl Div<&Value> for &Value {
    type Output = Value;

    fn div(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
            _ => panic!("Cannot divide {self:?} by {rhs:?}"),
        }
    }
}
