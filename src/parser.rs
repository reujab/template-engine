use crate::{
    error::ParseError,
    lexer::{Lexer, Operator, Token},
    value::Value,
};

/// The parser converts the tokens produced by the lexer into an abstract syntax tree.
pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    buffer: Vec<Token>,
}

#[derive(Debug)]
pub enum Node {
    Text(String),
    Number(f64),
    Variable(String),
    Operation(Box<Node>, Operator, Box<Node>),
    List(Box<Vec<Node>>),
    FunctionCall(String, Box<Vec<Node>>),
}

impl<'a> Parser<'a> {
    pub fn parse_input(input: &str) -> Result<Node, ParseError> {
        let mut lexer = Lexer::new(input);
        let parser = Parser::new(&mut lexer);
        parser.parse_all()
    }

    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        Self {
            lexer,
            buffer: Vec::with_capacity(1),
        }
    }

    pub fn parse(&mut self) -> Result<Option<Node>, ParseError> {
        let token = match self.next()? {
            None => return Ok(None),
            Some(token) => token,
        };
        let node = match token {
            Token::Text(string) => Node::Text(string),
            Token::OpenTemplate => self.parse_template()?,
            token => panic!("Something went horribly wrong: {token:?}"),
        };
        Ok(Some(node))
    }

    pub fn parse_all(mut self) -> Result<Node, ParseError> {
        let mut nodes = Vec::new();
        while let Some(node) = self.parse()? {
            nodes.push(node);
        }
        Ok(Node::List(nodes.into()))
    }

    fn parse_template(&mut self) -> Result<Node, ParseError> {
        // let expression = self.parse_expr()?;
        // self.expect(Token::CloseTemplate, "template")?;
        // Ok(expression)
        let node = match self.expect_next()? {
            Token::Identifier(identifier) => match self.expect_next()? {
                Token::OpeningParen => return self.parse_function_call(identifier),
                // Math expression starting with variable
                token => {
                    self.restore(token);
                    self.restore(Token::Identifier(identifier));
                    let expression = self.parse_expr()?;
                    self.expect(Token::CloseTemplate, "template identifier")?;
                    return Ok(expression);
                }
            },
            Token::Literal(literal) => match literal {
                // Math expression starting with number.
                Value::Number(num) => {
                    self.restore(Token::Literal(Value::Number(num)));
                    let expression = self.parse_expr()?;
                    self.expect(Token::CloseTemplate, "template literal number")?;
                    expression
                }
                // Strings
                Value::String(string) => match self.expect_next()? {
                    Token::Operator(Operator::Add) => {
                        Node::List(vec![Node::Text(string), self.parse_template()?].into())
                    }
                    Token::Operator(Operator::Multiply) => Node::Operation(
                        Node::Text(string).into(),
                        Operator::Multiply,
                        self.parse_template()?.into(),
                    ),
                    Token::CloseTemplate => Node::Text(string),
                    token => {
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "template literal string",
                        ))
                    }
                },
            },
            Token::OpeningParen => {
                self.restore(Token::OpeningParen);
                let expression = self.parse_expr()?;
                self.expect(Token::CloseTemplate, "template parentheses")?;
                expression
            }
            token => return Err(ParseError::UnexpectedToken(token, "template")),
        };
        Ok(node)
    }

    fn parse_function_call(&mut self, identifier: String) -> Result<Node, ParseError> {
        let mut args = Vec::new();
        loop {
            match self.expect_next()? {
                Token::ClosingParen => break,
                Token::Literal(Value::String(string)) => {
                    args.push(Node::Text(string));
                }
                token => {
                    self.restore(token);
                    args.push(self.parse_expr()?);
                }
            }
            match self.expect_next()? {
                Token::ClosingParen => break,
                Token::Comma => continue,
                token => return Err(ParseError::UnexpectedToken(token, "function call")),
            }
        }
        self.expect(Token::CloseTemplate, "function call")?;
        Ok(Node::FunctionCall(identifier, args.into()))
    }

    fn next(&mut self) -> Result<Option<Token>, ParseError> {
        if self.buffer.is_empty() {
            self.lexer
                .next_token()
                .map_err(|err| ParseError::LexerError(err))
        } else {
            Ok(self.buffer.pop())
        }
    }

    /// Call this function when you get a token you don't need.
    /// Call in reverse order: a = self.next(); b = self.next(); self.restore(b); self.restore(a);
    fn restore(&mut self, token: Token) {
        self.buffer.push(token);
    }

    /// This functions handles the lowest-precedence operations (plus and minus).
    fn parse_expr(&mut self) -> Result<Node, ParseError> {
        let mut expression = self.parse_term()?;
        while let Some(token) = self.next()? {
            match token {
                Token::Operator(operator) => match operator {
                    Operator::Add | Operator::Subtract => {
                        let term = self.parse_term()?;
                        expression = Node::Operation(expression.into(), operator, term.into());
                        continue;
                    }
                    // Remember, at this point, we've consumed any other operators. (At least for
                    // this term.)
                    _ => panic!("Something went horribly wrong."),
                },
                _ => self.restore(token),
            }
            break;
        }
        Ok(expression)
    }

    /// This functions parses a term and handles the higher-precedence operations (multiply and divide).
    fn parse_term(&mut self) -> Result<Node, ParseError> {
        let mut term = self.parse_factor()?;
        while let Some(token) = self.next()? {
            match token {
                Token::Operator(operator) => match operator {
                    Operator::Multiply | Operator::Divide => {
                        let factor = self.parse_factor()?;
                        term = Node::Operation(term.into(), operator, factor.into());
                        continue;
                    }
                    _ => self.restore(token),
                },
                _ => self.restore(token),
            }
            break;
        }
        Ok(term)
    }

    /// This function parses parentheses and literals.
    fn parse_factor(&mut self) -> Result<Node, ParseError> {
        let token = self.expect_next()?;
        let factor = match token {
            Token::Identifier(identifier) => Node::Variable(identifier),
            Token::Literal(Value::Number(num)) => Node::Number(num),
            Token::Literal(Value::String(string)) => Node::Text(string),
            Token::OpeningParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::ClosingParen, "factor parentheses")?;
                expr
            }
            token => return Err(ParseError::UnexpectedToken(token, "factor")),
        };
        Ok(factor)
    }

    fn expect_next(&mut self) -> Result<Token, ParseError> {
        self.next()?.ok_or(ParseError::UnexpectedEOF)
    }

    fn expect(&mut self, expected_token: Token, parsing: &'static str) -> Result<(), ParseError> {
        let next_token = self.expect_next()?;
        if expected_token == next_token {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(next_token, parsing))
        }
    }
}
