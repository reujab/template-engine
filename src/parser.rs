use crate::{
    error::ParseError,
    lexer::{Keyword, Lexer, Operator, Token},
    node::Node,
    value::Value,
};

/// The parser converts the tokens produced by the lexer into an abstract syntax tree.
pub struct Parser<'a> {
    pub(crate) lexer: &'a mut Lexer<'a>,
    pub(crate) buffer: Vec<Token>,
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
        while let Some(node) = self.next_node()? {
            nodes.push(node);
        }
        Ok(Node::Root(nodes))
    }

    pub fn next_node(&mut self) -> Result<Option<Node>, ParseError> {
        let token = match self.next_token()? {
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
        let node = match self.expect_next_token()? {
            Token::Keyword(Keyword::If) => self.parse_if()?,
            // These next two cases are returning early for expected "unexpected" tokens (parse_if).
            Token::Keyword(keyword) => {
                return Err(ParseError::UnexpectedToken(
                    Token::Keyword(keyword),
                    "template",
                ));
            }
            Token::Operator(Operator::Divide) => {
                return Err(ParseError::UnexpectedToken(
                    Token::Operator(Operator::Divide),
                    "template",
                ));
            }
            token => {
                self.restore(token);
                self.parse_expr()?
            }
        };
        self.expect(Token::CloseTemplate, "template")?;
        Ok(node)
    }

    fn parse_if(&mut self) -> Result<Node, ParseError> {
        let condition = self.parse_expr()?;
        self.expect(Token::CloseTemplate, "template if")?;
        let mut then_nodes = Vec::new();
        let else_node = loop {
            match self.next_node() {
                Ok(node) => then_nodes.push(node.ok_or(ParseError::UnexpectedEOF)?),
                Err(ParseError::UnexpectedToken(Token::Keyword(Keyword::Elif), _)) => {
                    // This is so beautiful. We just consumed the elif keyword, so this single call
                    // will parse the rest of this "if" tree up to the {{/if tokens, leaving the
                    // closing template token to be consumed by `parse_template`. Beautiful
                    // recursion.
                    break Some(self.parse_if()?);
                }
                Err(ParseError::UnexpectedToken(Token::Keyword(Keyword::Else), _)) => {
                    break Some(self.parse_else()?);
                }
                Err(ParseError::UnexpectedToken(Token::Operator(Operator::Divide), _)) => {
                    self.expect(Token::Keyword(Keyword::If), "end if")?;
                    break None;
                }
                Err(err) => return Err(err),
            }
        };
        let then_node = Node::Root(then_nodes.into());
        Ok(Node::IfThenElse(
            condition.into(),
            then_node.into(),
            else_node.map(Into::into),
        ))
    }

    fn parse_else(&mut self) -> Result<Node, ParseError> {
        self.expect(Token::CloseTemplate, "template else")?;
        let mut nodes = Vec::new();
        loop {
            match self.next_node() {
                Ok(node) => nodes.push(node.ok_or(ParseError::UnexpectedEOF)?),
                // {{ /if }}
                Err(ParseError::UnexpectedToken(Token::Operator(Operator::Divide), _)) => {
                    self.expect(Token::Keyword(Keyword::If), "else end if")?;
                    // We leave the CloseTemplate token for `parse_template` to consume.
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(Node::Root(nodes))
    }

    fn parse_expr(&mut self) -> Result<Node, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Node, ParseError> {
        let mut expression = self.parse_and()?;
        while let Some(token) = self.next_token()? {
            match token {
                Token::Operator(Operator::Or) => {
                    let rhs = self.parse_and()?;
                    expression = Node::Operation(expression.into(), Operator::Or, rhs.into());
                    continue;
                }
                _ => self.restore(token),
            }
            break;
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Node, ParseError> {
        let mut expression = self.parse_comparisons()?;
        while let Some(token) = self.next_token()? {
            match token {
                Token::Operator(Operator::And) => {
                    let rhs = self.parse_comparisons()?;
                    expression = Node::Operation(expression.into(), Operator::And, rhs.into());
                    continue;
                }
                _ => self.restore(token),
            }
            break;
        }
        Ok(expression)
    }

    fn parse_comparisons(&mut self) -> Result<Node, ParseError> {
        let mut expression = self.parse_polynomial()?;
        while let Some(token) = self.next_token()? {
            match token {
                Token::Operator(operator) => match operator {
                    Operator::IsEqualTo | Operator::IsNotEqualTo => {
                        let rhs = self.parse_polynomial()?;
                        expression = Node::Operation(expression.into(), operator, rhs.into());
                        continue;
                    }
                    _ => self.restore(Token::Operator(operator)),
                },
                _ => self.restore(token),
            }
            break;
        }
        Ok(expression)
    }

    /// This functions handles the lowest-precedence number operations (plus and minus).
    fn parse_polynomial(&mut self) -> Result<Node, ParseError> {
        let mut expression = self.parse_term()?;
        while let Some(token) = self.next_token()? {
            match token {
                Token::Operator(operator) => match operator {
                    Operator::Add | Operator::Subtract => {
                        let term = self.parse_term()?;
                        expression = Node::Operation(expression.into(), operator, term.into());
                        continue;
                    }
                    _ => self.restore(Token::Operator(operator)),
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
        while let Some(token) = self.next_token()? {
            match token {
                Token::Operator(operator) => match operator {
                    Operator::Multiply | Operator::Divide => {
                        let factor = self.parse_factor()?;
                        term = Node::Operation(term.into(), operator, factor.into());
                        continue;
                    }
                    _ => self.restore(Token::Operator(operator)),
                },
                _ => self.restore(token),
            }
            break;
        }
        Ok(term)
    }

    /// This function parses parentheses and literals.
    fn parse_factor(&mut self) -> Result<Node, ParseError> {
        let token = self.expect_next_token()?;
        let factor = match token {
            Token::Literal(value) => Node::Value(value),
            Token::OpeningParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::ClosingParen, "parentheses")?;
                expr
            }
            Token::Identifier(identifier) => match self.expect_next_token()? {
                Token::OpeningParen => self.parse_function_call(identifier)?,
                next_token => {
                    self.restore(next_token);
                    Node::Variable(identifier)
                }
            },
            Token::Exclamation => Node::Not(self.parse_factor()?.into()),
            token => return Err(ParseError::UnexpectedToken(token, "factor")),
        };
        Ok(factor)
    }

    fn parse_function_call(&mut self, identifier: String) -> Result<Node, ParseError> {
        let mut args = Vec::new();
        loop {
            match self.expect_next_token()? {
                Token::ClosingParen => break,
                token => {
                    self.restore(token);
                    args.push(self.parse_expr()?);
                }
            }
            match self.expect_next_token()? {
                Token::ClosingParen => break,
                Token::Comma => continue,
                token => return Err(ParseError::UnexpectedToken(token, "function call")),
            }
        }
        Ok(Node::FunctionCall(identifier, args))
    }
}
