use std::collections::{HashMap, HashSet};

use crate::{error::ValueError, lexer::Operator, value::Value};

#[derive(Debug)]
pub enum Node {
    Root(Vec<Node>),
    Value(Value),
    Variable(String),
    FunctionCall(String, Vec<Node>),
    Operation(Box<Node>, Operator, Box<Node>),
    IfThenElse(Box<Node>, Box<Node>, Option<Box<Node>>),
}

impl Node {
    pub fn evaluate(
        &self,
        variables: &HashMap<String, &Value>,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> Value>,
    ) -> Result<Value, ValueError> {
        match self {
            Node::Value(value) => Ok(value.clone()),
            Node::Variable(identifier) => {
                let variable = *variables
                    .get(identifier)
                    .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?;
                Ok(variable.clone())
            }
            Node::FunctionCall(identifier, args) => {
                let function = functions
                    .get(identifier)
                    .ok_or_else(|| ValueError::UndefinedVariable(identifier.clone()))?;
                let args = args
                    .iter()
                    .map(|node| node.evaluate(variables, functions))
                    .collect::<Result<Vec<Value>, ValueError>>()?;
                Ok((*function)(args))
            }
            Node::Root(nodes) => {
                let mut buffer = String::new();
                for node in &**nodes {
                    buffer += &node.evaluate(variables, functions)?.to_string();
                }
                Ok(Value::String(buffer))
            }
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs.evaluate(variables, functions)?;
                let rhs = rhs.evaluate(variables, functions)?;
                match op {
                    Operator::Multiply => &lhs * &rhs,
                    Operator::Divide => &lhs / &rhs,
                    Operator::Add => &lhs + &rhs,
                    Operator::Subtract => &lhs - &rhs,
                    Operator::IsEqualTo => Ok(Value::Boolean(lhs == rhs)),
                    Operator::And => Ok(Value::Boolean(lhs.is_truthy() && rhs.is_truthy())),
                    Operator::Or => Ok(Value::Boolean(lhs.is_truthy() || rhs.is_truthy())),
                }
            }
            Node::IfThenElse(condition, then_node, else_node) => {
                let evaluation = condition.evaluate(variables, functions)?;
                if evaluation.is_truthy() {
                    then_node.evaluate(variables, functions)
                } else if let Some(else_node) = else_node {
                    else_node.evaluate(variables, functions)
                } else {
                    Ok(Value::String(String::new()))
                }
            }
        }
    }

    pub fn referenced_vars(&self) -> HashSet<&String> {
        let mut references = HashSet::new();
        match self {
            Node::Root(nodes) => {
                for node in &**nodes {
                    references.extend(node.referenced_vars());
                }
            }
            Node::FunctionCall(_identifier, args) => {
                for node in &**args {
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
            Node::Variable(identifier) => {
                references.insert(identifier);
            }
            Node::Value(_) => {}
        }
        references
    }
}
