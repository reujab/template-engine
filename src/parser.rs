use crate::{
    error::ParseError,
    lexer::{Lexer, Operator, Token},
    value::Value,
};

/// The parser converts the tokens produced by the lexer into an abstract syntax tree.
pub struct Parser<'a> {
    pub(crate) lexer: &'a mut Lexer<'a>,
    pub(crate) buffer: Vec<Token>,
}

#[derive(Debug)]
pub enum Node {
    Value(Value),
    Variable(String),
    FunctionCall(String, Box<Vec<Node>>),
    Operation(Box<Node>, Operator, Box<Node>),
    List(Box<Vec<Node>>),
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

    pub fn parse_all(mut self) -> Result<Node, ParseError> {
        let mut nodes = Vec::new();
        while let Some(node) = self.parse()? {
            nodes.push(node);
        }
        Ok(Node::List(nodes.into()))
    }

    pub fn parse(&mut self) -> Result<Option<Node>, ParseError> {
        let token = match self.next()? {
            None => return Ok(None),
            Some(token) => token,
        };
        let node = match token {
            Token::Text(string) => Node::Value(Value::String(string)),
            Token::OpenTemplate => self.parse_template()?,
            token => panic!("Something went horribly wrong: {token:?}"),
        };
        Ok(Some(node))
    }

    fn parse_template(&mut self) -> Result<Node, ParseError> {
        let expression = self.parse_expr()?;
        self.expect(Token::CloseTemplate, "template")?;
        Ok(expression)
    }

    /// This functions handles the lowest-precedence number operations (plus and minus).
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
                    _ => self.restore(token),
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
            Token::Identifier(identifier) => match self.expect_next()? {
                Token::OpeningParen => self.parse_function_call(identifier)?,
                t => {
                    self.restore(t);
                    Node::Variable(identifier)
                }
            },
            Token::Literal(value) => Node::Value(value),
            Token::OpeningParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::ClosingParen, "factor parentheses")?;
                expr
            }
            token => return Err(ParseError::UnexpectedToken(token, "factor")),
        };
        Ok(factor)
    }

    fn parse_function_call(&mut self, identifier: String) -> Result<Node, ParseError> {
        let mut args = Vec::new();
        loop {
            match self.expect_next()? {
                Token::ClosingParen => break,
                Token::Literal(Value::String(string)) => {
                    args.push(Node::Value(Value::String(string)));
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
        Ok(Node::FunctionCall(identifier, args.into()))
    }
}
