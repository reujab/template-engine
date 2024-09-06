use std::collections::HashMap;

use parser::Parser;
use value::Value;

mod error;
mod evaluate;
mod lexer;
mod parser;
mod parser_helpers;
mod value;

fn main() {
    let node = Parser::parse_input("{{ 3 + 3  + '3' }}").unwrap();
    println!("{node:?}");
    let mut variables = HashMap::new();
    variables.insert("a".to_owned(), Value::Number(8.0));
    variables.insert("b".to_owned(), Value::Number(2.0));
    let mut functions = HashMap::new();
    let exec = |_vec: Vec<Value>| Value::Number(42.0);
    // functions.insert("exec".to_owned(), Box::new(exec));
    functions.insert("exec".to_owned(), exec);
    println!("{:?}", node.evaluate(&variables, &functions).unwrap());
}
