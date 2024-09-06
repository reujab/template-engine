use std::collections::HashMap;

use parser::Parser;
use value::Value;

mod error;
mod evaluate;
mod lexer;
mod parser;
mod value;

fn main() {
    let node = Parser::parse_input("{{'test'*a/b}}").unwrap();
    println!("{node:?}");
    let mut variables = HashMap::new();
    variables.insert("a".to_owned(), Value::Number(8.0));
    variables.insert("b".to_owned(), Value::Number(2.0));
    println!("{:?}", node.evaluate(&variables));
}
