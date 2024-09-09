use std::collections::{HashMap, HashSet};

use crate::{error::ValueError, lexer::Operator, value::OwnedValue, variables::Variables, Value};

#[derive(Debug)]
pub enum Node {
    Body(Vec<Node>),
    Value(OwnedValue),
    Variable(String),
    FunctionCall(String, Vec<Node>),
    Array(Vec<Node>),
    Operation(Box<Node>, Operator, Box<Node>),
    Not(Box<Node>),
    Negate(Box<Node>),
    IfThenElse(Box<Node>, Box<Node>, Option<Box<Node>>),
    /// The first field is the identifier. The second field is the array. The third field is the
    /// body. The fourth field is the separator.
    ForIn(String, Box<Node>, Box<Node>, Option<Box<Node>>),
}

impl Node {
    pub fn evaluate<V: Variables>(
        &self,
        variables: &V,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> OwnedValue>,
    ) -> Result<String, ValueError> {
        let body = self._evaluate(variables, functions, &HashMap::new())?;
        Ok(body.unwrap_string())
    }

    fn _evaluate<'a, V: Variables>(
        &'a self,
        variables: &'a V,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> OwnedValue>,
        local_vars: &HashMap<String, Value<'a>>,
    ) -> Result<Value, ValueError> {
        match self {
            Node::Body(nodes) => {
                let mut buffer = String::new();
                for node in &**nodes {
                    let eval_value = node._evaluate(variables, functions, local_vars)?;
                    let value = eval_value.inner();
                    // This cannot be turned into a method: Reference to temporary value dropped.
                    let string = match value {
                        OwnedValue::String(string) => string,
                        a => &a.to_string(),
                    };
                    buffer += string;
                }
                Ok(Value::Owned(OwnedValue::String(buffer)))
            }
            Node::Value(value) => Ok(Value::Borrowed(value)),
            Node::Variable(identifier) => {
                let variable = match local_vars.get(identifier) {
                    None => Value::Borrowed(
                        variables
                            .get(identifier)
                            .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?,
                    ),
                    Some(Value::Borrowed(value)) => Value::Borrowed(*value),
                    // Data within `local_vars` will always be borrowed.
                    _ => panic!(),
                };
                Ok(variable)
            }
            Node::FunctionCall(identifier, args) => {
                let function = functions
                    .get(identifier)
                    .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?;
                let args = args
                    .iter()
                    .map(|node| node._evaluate(variables, functions, local_vars))
                    .collect::<Result<Vec<Value>, ValueError>>()?;
                let result = (*function)(args);
                Ok(result.into())
            }

            Node::Array(nodes) => {
                let array = nodes
                    .iter()
                    .map(|node| {
                        node._evaluate(variables, functions, local_vars)
                            .map(|value| value.to_owned_value())
                    })
                    .collect::<Result<Vec<OwnedValue>, ValueError>>()?;
                Ok(array.into())
            }
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs._evaluate(variables, functions, local_vars)?;
                let rhs = rhs._evaluate(variables, functions, local_vars)?;
                let lhs = lhs.inner();
                let rhs = rhs.inner();
                match op {
                    Operator::Multiply => (lhs * rhs).map(Into::into),
                    Operator::Divide => (lhs / rhs).map(Into::into),
                    Operator::Add => (lhs + rhs).map(Into::into),
                    Operator::Subtract => (lhs - rhs).map(Into::into),
                    Operator::IsEqualTo => Ok((lhs == rhs).into()),
                    Operator::IsNotEqualTo => Ok((lhs != rhs).into()),
                    Operator::And => Ok((lhs.is_truthy() && rhs.is_truthy()).into()),
                    Operator::Or => Ok((lhs.is_truthy() || rhs.is_truthy()).into()),
                }
            }
            Node::Not(node) => Ok((!node
                ._evaluate(variables, functions, local_vars)?
                .inner()
                .is_truthy())
            .into()),
            Node::Negate(node) => Ok((-node
                ._evaluate(variables, functions, local_vars)?
                .unwrap_f64()?)
            .into()),
            Node::IfThenElse(condition, then_node, else_node) => {
                let evaluation = condition._evaluate(variables, functions, local_vars)?;
                let condition_value = evaluation.inner();
                if condition_value.is_truthy() {
                    then_node._evaluate(variables, functions, local_vars)
                } else if let Some(else_node) = else_node {
                    else_node._evaluate(variables, functions, local_vars)
                } else {
                    Ok(String::new().into())
                }
            }
            Node::ForIn(identifier, array, body, separator) => {
                let evaluation = array._evaluate(variables, functions, local_vars)?;
                let array = match evaluation.inner() {
                    OwnedValue::Array(array) => array,
                    value => return Err(ValueError::IterateError(value.clone())),
                };
                let separator = match separator {
                    None => "",
                    Some(separator) => {
                        match separator._evaluate(variables, functions, local_vars)? {
                            Value::Borrowed(value) => match value {
                                OwnedValue::String(string) => string,
                                _ => {
                                    return Err(ValueError::OperationError(
                                        "Invalid separator.".into(),
                                    ));
                                }
                            },
                            _ => panic!(),
                        }
                    }
                };
                let mut local_vars = local_vars.clone();
                let mut buffer = String::new();
                for (i, item) in array.iter().enumerate() {
                    local_vars.insert(identifier.to_owned(), Value::Borrowed(item));
                    let evaluation = body._evaluate(variables, functions, &local_vars)?;
                    let body = evaluation.inner();
                    let string = match body {
                        OwnedValue::String(string) => string,
                        value => &value.to_string(),
                    };
                    buffer += string;
                    if i != array.len() - 1 {
                        buffer += &separator;
                    }
                }
                Ok(buffer.into())
            }
        }
    }

    pub fn referenced_vars(&self) -> HashSet<&String> {
        let mut references = HashSet::new();
        match self {
            Node::Body(nodes) => {
                for node in &**nodes {
                    references.extend(node.referenced_vars());
                }
            }
            Node::FunctionCall(_identifier, args) => {
                for node in &**args {
                    references.extend(node.referenced_vars());
                }
            }
            Node::Array(array) => {
                for node in array {
                    references.extend(node.referenced_vars());
                }
            }
            Node::Operation(lhs, _, rhs) => {
                references.extend(lhs.referenced_vars());
                references.extend(rhs.referenced_vars());
            }
            Node::IfThenElse(condition, then_node, else_node) => {
                references.extend(condition.referenced_vars());
                references.extend(then_node.referenced_vars());
                if let Some(else_node) = else_node {
                    references.extend(else_node.referenced_vars());
                }
            }
            Node::ForIn(identifier, array, body, separator) => {
                references.extend(body.referenced_vars());
                if let Some(separator) = separator {
                    references.extend(separator.referenced_vars());
                }
                references.remove(identifier);
                references.extend(array.referenced_vars());
            }
            Node::Variable(identifier) => {
                references.insert(identifier);
            }
            Node::Not(node) => {
                references.extend(node.referenced_vars());
            }
            Node::Negate(node) => {
                references.extend(node.referenced_vars());
            }
            Node::Value(_) => {}
        }
        references
    }
}
