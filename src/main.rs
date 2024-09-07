mod error;
mod lexer;
mod math;
mod node;
mod parser;
mod parser_helpers;
mod value;

use std::collections::HashMap;

use parser::Parser;
use value::Value;

fn main() {
    let input = "{{for a in range(5)}}{{for b in range(a)}}{{b}}{{/for}} {{/for}}";
    let node = Parser::parse_input(input).unwrap();

    let mut variables = HashMap::new();
    variables.insert("a".into(), &Value::Number(4.0));
    let mut functions = HashMap::new();
    functions.insert("range".to_owned(), |args: Vec<Value>| {
        let (lower_bound, upper_bound) = match args.len() {
            1 => (1, as_usize(&args[0])),
            2 => (as_usize(&args[0]), as_usize(&args[1])),
            _ => unimplemented!(),
        };
        assert!(lower_bound <= upper_bound);
        let range = (lower_bound..=upper_bound)
            .map(|n| Value::Number(n as f64))
            .collect::<Vec<Value>>();
        Value::Array(range)
    });
    node.referenced_vars();
    println!("{:?}", node.evaluate(&variables, &functions).unwrap());
}

fn as_usize(value: &Value) -> usize {
    match value {
        Value::Number(num) => *num as usize,
        _ => unimplemented!(),
    }
}
