use crate::{
    error::ParseError,
    lexer::{Operator, Token},
    node::Node,
    parser::Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn parse_expr(&mut self) -> Result<Node, ParseError> {
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
            Token::OpeningSquareBracket => self.parse_array()?,
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

    fn parse_array(&mut self) -> Result<Node, ParseError> {
        let mut array = Vec::new();
        loop {
            match self.expect_next_token()? {
                Token::ClosingSquareBracket => break,
                token => {
                    self.restore(token);
                    array.push(self.parse_expr()?);
                }
            }
            match self.expect_next_token()? {
                Token::ClosingSquareBracket => break,
                Token::Comma => continue,
                token => return Err(ParseError::UnexpectedToken(token, "array")),
            }
        }
        Ok(Node::Array(array))
    }
}
