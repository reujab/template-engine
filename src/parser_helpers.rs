use crate::{error::ParseError, lexer::Token, parser::Parser};

impl<'a> Parser<'a> {
    pub(crate) fn next_token(&mut self) -> Result<Option<Token>, ParseError> {
        if self.buffer.is_empty() {
            self.lexer
                .next_token()
                .map_err(|err| ParseError::LexerError(err))
        } else {
            Ok(self.buffer.pop())
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

    /// Call this function when you get a token you don't need.
    /// Call in reverse order: a = self.next(); b = self.next(); self.restore(b); self.restore(a);
    pub(crate) fn restore(&mut self, token: Token) {
        self.buffer.push(token);
    }
}
