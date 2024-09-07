use std::collections::{HashMap, HashSet};

use crate::{error::ValueError, lexer::Operator, value::Value};

#[derive(Debug)]
pub enum Node {
    Body(Vec<Node>),
    Value(Value),
    Variable(String),
    FunctionCall(String, Vec<Node>),
    Array(Vec<Node>),
    Operation(Box<Node>, Operator, Box<Node>),
    Not(Box<Node>),
    IfThenElse(Box<Node>, Box<Node>, Option<Box<Node>>),
    ForIn(String, Box<Node>, Box<Node>),
}

impl Node {
    pub fn evaluate(
        &self,
        variables: &HashMap<String, &Value>,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> Value>,
    ) -> Result<Value, ValueError> {
        self._evaluate(variables, functions, &HashMap::new())
    }

    fn _evaluate(
        &self,
        variables: &HashMap<String, &Value>,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> Value>,
        local_vars: &HashMap<String, Value>,
    ) -> Result<Value, ValueError> {
        match self {
            Node::Body(nodes) => {
                let mut buffer = String::new();
                for node in &**nodes {
                    buffer += &node
                        ._evaluate(variables, functions, local_vars)?
                        .to_string();
                }
                Ok(Value::String(buffer))
            }
            Node::Value(value) => Ok(value.clone()),
            Node::Variable(identifier) => {
                let variable = local_vars
                    .get(identifier)
                    .or_else(|| variables.get(identifier).copied())
                    .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?;
                Ok(variable.clone())
            }
            Node::FunctionCall(identifier, args) => {
                let function = functions
                    .get(identifier)
                    .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?;
                let args = args
                    .iter()
                    .map(|node| node._evaluate(variables, functions, local_vars))
                    .collect::<Result<Vec<Value>, ValueError>>()?;
                Ok((*function)(args))
            }
            Node::Array(nodes) => {
                // let array = Vec::with_capacity(nodes.len())
                let array = nodes
                    .iter()
                    .map(|node| node._evaluate(variables, functions, local_vars))
                    .collect::<Result<Vec<Value>, _>>()?;
                Ok(Value::Array(array))
            }
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs._evaluate(variables, functions, local_vars)?;
                let rhs = rhs._evaluate(variables, functions, local_vars)?;
                match op {
                    Operator::Multiply => &lhs * &rhs,
                    Operator::Divide => &lhs / &rhs,
                    Operator::Add => &lhs + &rhs,
                    Operator::Subtract => &lhs - &rhs,
                    Operator::IsEqualTo => Ok(Value::Boolean(lhs == rhs)),
                    Operator::IsNotEqualTo => Ok(Value::Boolean(lhs != rhs)),
                    Operator::And => Ok(Value::Boolean(lhs.is_truthy() && rhs.is_truthy())),
                    Operator::Or => Ok(Value::Boolean(lhs.is_truthy() || rhs.is_truthy())),
                }
            }
            Node::Not(node) => Ok(Value::Boolean(
                !node
                    ._evaluate(variables, functions, local_vars)?
                    .is_truthy(),
            )),
            Node::IfThenElse(condition, then_node, else_node) => {
                let evaluation = condition._evaluate(variables, functions, local_vars)?;
                if evaluation.is_truthy() {
                    then_node._evaluate(variables, functions, local_vars)
                } else if let Some(else_node) = else_node {
                    else_node._evaluate(variables, functions, local_vars)
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            Node::ForIn(identifier, array, body) => {
                let array = match array._evaluate(variables, functions, local_vars)? {
                    Value::Array(array) => array,
                    value => return Err(ValueError::IterateError(value)),
                };
                let mut local_vars = local_vars.clone();
                let mut buffer = String::new();
                for item in array {
                    local_vars.insert(identifier.clone(), item);
                    buffer += &body
                        ._evaluate(variables, functions, &local_vars)?
                        .to_string();
                }
                Ok(Value::String(buffer))
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
            Node::ForIn(identifier, array, body) => {
                references.extend(array.referenced_vars());
                references.extend(body.referenced_vars());
                references.remove(identifier);
            }
            Node::Variable(identifier) => {
                references.insert(identifier);
            }
            Node::Not(node) => {
                references.extend(node.referenced_vars());
            }
            Node::Value(_) => {}
        }
        references
    }
}
