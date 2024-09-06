use std::collections::HashMap;

use crate::{error::ValueError, lexer::Operator, parser::Node, value::Value};

impl Node {
    pub fn evaluate(
        &self,
        variables: &HashMap<String, Value>,
        functions: &HashMap<String, impl Fn(Vec<Value>) -> Value>,
    ) -> Result<Value, ValueError> {
        match self {
            Node::Value(value) => Ok(value.clone()),
            Node::FunctionCall(identifier, args) => {
                let function = &functions[identifier];
                let args = args
                    .iter()
                    .map(|node| node.evaluate(variables, functions))
                    .collect::<Result<Vec<Value>, ValueError>>()?;
                Ok((*function)(args))
            }
            Node::List(nodes) => {
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
            Node::Variable(identifier) => Ok(variables[identifier].clone()),
        }
    }
}
