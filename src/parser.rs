use crate::lexer::{Lexer, Operator, Token};

/// The parser converts the tokens produced by the lexer into an abstract syntax tree.
pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    buffer: Vec<Token>,
}

#[derive(Debug)]
pub enum Node {
    Literal(f64),
    Operation(Box<Node>, Operator, Box<Node>),
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        Self {
            lexer,
            buffer: Vec::with_capacity(1),
        }
    }

    pub fn parse(&mut self) -> Node {
        self.parse_expr()
    }

    fn next(&mut self) -> Option<Token> {
        self.buffer.pop().or_else(|| self.lexer.next())
    }

    /// Call this function when you get a token you don't need.
    /// Call in reverse order: a = self.next(); b = self.next(); self.restore(b); self.restore(a);
    fn restore(&mut self, token: Token) {
        self.buffer.push(token);
    }

    /// This is our first parse function. It handles the lowest-precedence operations (plus and minus).
    /// This could be rewritten with precedence climbing, but I prefer clear code over concise code.
    fn parse_expr(&mut self) -> Node {
        // (1) When we are parsing an expression, we want to get the first "thing" whether that's
        // a literal number or an operation inside parentheses.
        let mut expression = self.parse_term();
        // (5) So now we have our `term`, 3, and the next token is a plus operator.
        loop {
            let token = match self.next() {
                None => break,
                Some(token) => token,
            };
            match token {
                Token::Operator(operator) => match operator {
                    // (6) We found the plus operator, now we need to parse the right-hand side.
                    // We know the right-hand side is a term, so we call `parse_term`.
                    Operator::Add | Operator::Subtract => {
                        let term = self.parse_term();
                        expression = Node::Operation(expression.into(), operator, term.into());
                        continue;
                    }
                    _ => self.restore(token),
                },
                _ => self.restore(token),
            }
            break;
        }
        expression
    }

    /// This functions parses a term and handles the higher-precedence operations (multiply and divide).
    fn parse_term(&mut self) -> Node {
        // (2) One more recursion. We'll come back to this later.
        let mut term = self.parse_factor();
        // (4) Now we started building our term (in this case, 3). The next token is an addition
        // operator, so this term is finished. Return 3.
        // (7) So we already parsed `3 +`, and just now we parsed the `8`. Now, our remaining tokens
        // are `[Op(Divide), Literal(2)]`, and this time, we have division.
        loop {
            let token = match self.next() {
                None => break,
                Some(token) => token,
            };
            match token {
                Token::Operator(operator) => match operator {
                    // (8) Here, we add the next factor (2).
                    Operator::Multiply | Operator::Divide => {
                        let factor = self.parse_factor();
                        term = Node::Operation(term.into(), operator, factor.into());
                        continue;
                    }
                    _ => self.restore(token),
                },
                _ => self.restore(token),
            }
            break;
        }
        term
    }

    /// This function parses parentheses and literals.
    fn parse_factor(&mut self) -> Node {
        // (3) Here, we're getting that "thing" from earlier. For this example, let's say we're
        // parsing `3 + 8 / 2`. This function will get the first token (3) and return it.
        let token = self.expect_next();
        match token {
            Token::Number(num) => Node::Literal(num),
            Token::OpeningParen => {
                let expr = self.parse_expr();
                self.expect(Token::ClosingParen);
                expr
            }
            token => panic!("unexpected token {token:?}"),
        }
    }

    fn expect_next(&mut self) -> Token {
        self.next().ok_or("unexpected EOF").unwrap()
    }

    fn expect(&mut self, token: Token) {
        if token != self.expect_next() {
            panic!("expected {token:?}");
        }
    }
}
