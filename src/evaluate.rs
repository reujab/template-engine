use std::collections::HashMap;

use crate::{lexer::Operator, parser::Node, value::Value};

impl Node {
    pub fn evaluate(&self, variables: &HashMap<String, Value>) -> Value {
        match self {
            Node::FunctionCall(_identifier, _args) => unimplemented!(),
            Node::List(nodes) => {
                let mut buffer = String::new();
                for node in &**nodes {
                    buffer += &node.evaluate(variables).to_string();
                }
                Value::String(buffer)
            }
            Node::Number(num) => Value::Number(*num),
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs.evaluate(variables);
                let rhs = rhs.evaluate(variables);
                match op {
                    Operator::Add => &lhs + &rhs,
                    Operator::Subtract => &lhs - &rhs,
                    Operator::Multiply => &lhs * &rhs,
                    Operator::Divide => &lhs / &rhs,
                }
            }
            Node::Text(string) => Value::String(string.to_owned()),
            Node::Variable(identifier) => variables[identifier].clone(),
        }
    }
}
