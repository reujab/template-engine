mod error;
mod lexer;
mod node;
mod parser;
mod parser_helpers;
mod value;

use std::collections::HashMap;

use parser::Parser;
use value::Value;

fn main() {
    let node = Parser::parse_input(
        "{{if a == b}}{{if b}}a an{{b}}d b{{/if}} or just a{{elif b}}b{{else}}!a and !b{{/if}}",
    )
    .unwrap();
    println!("{node:#?}");
    let mut variables = HashMap::new();
    let a = Value::String("8.0".into());
    let b = Value::String("8.0".into());
    variables.insert("a".to_owned(), &a);
    variables.insert("b".to_owned(), &b);
    let mut functions = HashMap::new();
    let exec = |_| Value::Number(42.0);
    functions.insert("exec".to_owned(), Box::new(exec));
    // functions.insert("exec".to_owned(), exec);
    // println!("{:?}", node.referenced_vars());
    let references = node.referenced_vars();
    println!("{references:?}");
    println!("{:?}", node.evaluate(&variables, &functions).unwrap());
}
