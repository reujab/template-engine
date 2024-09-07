use crate::{
    error::ParseError,
    lexer::{Keyword, Lexer, Operator, Token},
    node::Node,
    value::Value,
};

/// The parser converts the tokens produced by the lexer into an abstract syntax tree.
pub struct Parser<'a> {
    pub(crate) lexer: &'a mut Lexer<'a>,
    pub(crate) buffer: Option<Token>,
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
            buffer: None,
        }
    }

    pub fn parse_all(mut self) -> Result<Node, ParseError> {
        let mut nodes = Vec::new();
        while let Some(node) = self.next_node()? {
            nodes.push(node);
        }
        Ok(Node::Body(nodes))
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
            Token::Keyword(Keyword::For) => self.parse_for()?,
            // These next two cases are returning early for expected "unexpected" tokens.
            Token::Keyword(keyword) => {
                return Err(ParseError::ExpectedToken(Token::Keyword(keyword)));
            }
            Token::Operator(Operator::Divide) => {
                return Err(ParseError::ExpectedToken(Token::Operator(Operator::Divide)));
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
                Err(ParseError::ExpectedToken(Token::Keyword(Keyword::Elif))) => {
                    // This is so beautiful. We just consumed the elif keyword, so this single call
                    // will parse the rest of this "if" tree up to the {{/if tokens, leaving the
                    // closing template token to be consumed by `parse_template`. Beautiful
                    // recursion.
                    break Some(self.parse_if()?);
                }
                Err(ParseError::ExpectedToken(Token::Keyword(Keyword::Else))) => {
                    break Some(self.parse_else()?);
                }
                Err(ParseError::ExpectedToken(Token::Operator(Operator::Divide))) => {
                    self.expect(Token::Keyword(Keyword::If), "end if")?;
                    break None;
                }
                Err(err) => return Err(err),
            }
        };
        let then_node = Node::Body(then_nodes);
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
        Ok(Node::Body(nodes))
    }

    fn parse_for(&mut self) -> Result<Node, ParseError> {
        let identifier = match self.expect_next_token()? {
            Token::Identifier(identifier) => identifier,
            token => return Err(ParseError::UnexpectedToken(token, "for identifier")),
        };
        self.expect(Token::Keyword(Keyword::In), "for in")?;
        let array = self.parse_expr()?;
        self.expect(Token::CloseTemplate, "for array")?;

        let mut body = Vec::new();
        loop {
            match self.next_node() {
                Ok(node) => body.push(node.ok_or(ParseError::UnexpectedEOF)?),
                Err(ParseError::ExpectedToken(Token::Operator(Operator::Divide))) => {
                    self.expect(Token::Keyword(Keyword::For), "for")?;
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        let body_node = Node::Body(body);
        Ok(Node::ForIn(identifier, array.into(), body_node.into()))
    }
}
