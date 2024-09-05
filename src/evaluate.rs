use crate::{lexer::Operator, parser::Node};

impl Node {
    pub fn evaluate(&self) -> f64 {
        match self {
            Node::Literal(num) => *num,
            Node::Operation(lhs, op, rhs) => {
                let lhs = lhs.evaluate();
                let rhs = rhs.evaluate();
                match op {
                    Operator::Add => lhs + rhs,
                    Operator::Subtract => lhs - rhs,
                    Operator::Multiply => lhs * rhs,
                    Operator::Divide => lhs / rhs,
                }
            }
        }
    }
}
