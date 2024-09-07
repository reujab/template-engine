use std::collections::{HashMap, HashSet};

use crate::{error::ValueError, lexer::Operator, value::Value};

#[derive(Debug)]
pub enum Node {
    Root(Box<Vec<Node>>),
    Value(Value),
    Variable(String),
    FunctionCall(String, Box<Vec<Node>>),
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
                let function = &functions[identifier];
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
                    Operator::Add => Ok(&lhs + &rhs),
                    Operator::Subtract => &lhs - &rhs,
                    Operator::Multiply => &lhs * &rhs,
                    Operator::Divide => &lhs / &rhs,
                }
            }
            Node::IfThenElse(condition, then_node, else_node) => {
                let condition = condition.evaluate(variables, functions)?.is_truthy();
                if condition {
                    then_node.evaluate(variables, functions)
                } else if let Some(else_node) = else_node {
                    else_node.evaluate(variables, functions)
                } else {
                    Ok(Value::Number(0.0))
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
