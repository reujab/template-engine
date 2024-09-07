use crate::{error::ParseError, lexer::Token, parser::Parser};

impl<'a> Parser<'a> {
    pub(crate) fn next_token(&mut self) -> Result<Option<Token>, ParseError> {
        match &mut self.buffer {
            None => self.lexer.yield_token().map_err(ParseError::LexerError),
            buffer => Ok(buffer.take()),
        }
    }

    pub(crate) fn expect_next_token(&mut self) -> Result<Token, ParseError> {
        self.next_token()?.ok_or(ParseError::UnexpectedEOF)
    }

    pub(crate) fn expect(
        &mut self,
        expected_token: Token,
        parsing: &'static str,
    ) -> Result<(), ParseError> {
        let next_token = self.expect_next_token()?;
        if expected_token == next_token {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(next_token, parsing))
        }
    }

    /// Call this function when you get a token you don't need. Panics if you restore multiple
    /// tokens in a row.
    pub(crate) fn restore(&mut self, token: Token) {
        assert_eq!(self.buffer, None);
        self.buffer = Some(token);
    }
}
