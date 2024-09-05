use lexer::Lexer;
use parser::Parser;

mod evaluate;
mod lexer;
mod parser;

fn main() {
    let input = "1 + 8 / 2 - 3 * 2 * (5 - 3)"; // -7
    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let node = parser.parse();
    let result = node.evaluate();
    println!("{node:?}\n{result}");
    // If we were parsing a scripting language, we would keep parsing nodes. Since we're just
    // evaluating an expression, we only need one node.
}
