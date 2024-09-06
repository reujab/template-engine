use std::collections::HashMap;

use crate::{error::ValueError, lexer::Operator, parser::Node, value::Value};

impl Node {
    pub fn evaluate(&self, variables: &HashMap<String, Value>) -> Result<Value, ValueError> {
        match self {
            Node::Value(value) => Ok(value.clone()),
            Node::FunctionCall(_identifier, _args) => unimplemented!(),
            Node::List(nodes) => {
                let mut buffer = String::new();
                for node in &**nodes {
                    buffer += &node.evaluate(variables)?.to_string();
                }
                Ok(Value::String(buffer))
            }
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs.evaluate(variables)?;
                let rhs = rhs.evaluate(variables)?;
                match op {
                    Operator::Add => Ok(&lhs + &rhs),
                    Operator::Subtract => &lhs - &rhs,
                    Operator::Multiply => &lhs * &rhs,
                    Operator::Divide => &lhs / &rhs,
                }
            }
            Node::Variable(identifier) => Ok(variables[identifier].clone()),
        }
    }
}
