use crate::{error::LexerError, value::OwnedValue};

/// The lexer (a.k.a tokenizer) is responsible for converting the input into a one-dimensional
/// series of tokens. For example, the input `3 + (4/2)` into the lexer would yield:
/// `[Literal(3], Op(Add), OpenParen, Literal(4), Op(Divide) Literal(2), ClosingParen]`
/// This implementation uses minimal allocation.
pub struct Lexer<'a> {
    /// There are many different ways of representing the input: String, &str, Vec<char>, Chars,
    /// etc. A string slice works best here because we don't have to keep converting between
    /// different types. If we need the next character, we can just use
    /// `src[cursor..].chars().next()`, which is cheap because `.chars()` returns a lazy iterator.
    src: &'a str,
    token_start_byte: usize,
    cursor: usize,
    is_inside_template: bool,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Text(String),

    TemplateOpen,
    TemplateClose,

    OpeningParen,
    ClosingParen,

    OpeningSqBracket,
    ClosingSqBracket,

    Comma,
    Exclamation,

    Keyword(Keyword),
    Identifier(String),
    Literal(OwnedValue),
    Operator(Operator),
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    If,
    Elif,
    Else,

    For,
    In,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Multiply,
    Divide,
    Add,
    Subtract,
    IsEqualTo,
    IsNotEqualTo,
    And,
    Or,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            token_start_byte: 0,
            cursor: 0,
            is_inside_template: false,
        }
    }

    pub fn yield_token(&mut self) -> Result<Option<Token>, LexerError> {
        let next_char = match self.get_next_char() {
            None => return Ok(None),
            Some(next_char) => next_char,
        };

        let token = match next_char {
            '{' if !self.is_inside_template => {
                if self.get_if_is('{').is_none() {
                    // False alarm; return next string.
                    return self.yield_token();
                }
                self.is_inside_template = true;
                Token::TemplateOpen
            }
            '}' if self.is_inside_template => {
                self.expect_char('}')?;
                self.is_inside_template = false;
                Token::TemplateClose
            }
            'a'..='z' | 'A'..='Z' | '_' if self.is_inside_template => self.yield_identifier(),
            '0'..='9' | '.' if self.is_inside_template => self.yield_number()?,
            '"' | '\'' => self.yield_string(next_char)?,
            '(' if self.is_inside_template => Token::OpeningParen,
            ')' if self.is_inside_template => Token::ClosingParen,
            '[' if self.is_inside_template => Token::OpeningSqBracket,
            ']' if self.is_inside_template => Token::ClosingSqBracket,
            '*' if self.is_inside_template => Token::Operator(Operator::Multiply),
            '/' if self.is_inside_template => Token::Operator(Operator::Divide),
            '+' if self.is_inside_template => Token::Operator(Operator::Add),
            '-' if self.is_inside_template => Token::Operator(Operator::Subtract),
            ',' if self.is_inside_template => Token::Comma,
            '=' if self.is_inside_template => {
                self.expect_char('=')?;
                Token::Operator(Operator::IsEqualTo)
            }
            '!' if self.is_inside_template => match self.expect_next_char()? {
                '=' => Token::Operator(Operator::IsNotEqualTo),
                _ => {
                    self.backup();
                    Token::Exclamation
                }
            },
            '&' if self.is_inside_template => {
                self.expect_char('&')?;
                Token::Operator(Operator::And)
            }
            '|' if self.is_inside_template => {
                self.expect_char('|')?;
                Token::Operator(Operator::Or)
            }
            c if self.is_inside_template && c.is_whitespace() => {
                self.consume_whitespace();
                return self.yield_token();
            }
            c if self.is_inside_template => return Err(LexerError::UnexpectedCharacter(c)),
            _ => {
                // Note: This will cause any single curly brace to start its own text node. This
                // currently has no side effects.
                self.advance_while(|c| c != '{');
                Token::Text(self.get_slice().to_owned())
            }
        };
        self.end_token();
        Ok(Some(token))
    }

    fn get_next_char(&mut self) -> Option<char> {
        let next_char = self.peek();
        if let Some(next_char) = next_char {
            self.cursor += next_char.len_utf8();
        }
        next_char
    }

    fn expect_next_char(&mut self) -> Result<char, LexerError> {
        match self.get_next_char() {
            Some(c) => Ok(c),
            None => Err(LexerError::UnexpectedEOF),
        }
    }

    fn expect_char(&mut self, expected_char: char) -> Result<(), LexerError> {
        let next_char = self.expect_next_char()?;
        if next_char == expected_char {
            Ok(())
        } else {
            Err(LexerError::UnexpectedCharacter(next_char))
        }
    }

    fn get_if_is(&mut self, c: char) -> Option<char> {
        let next_char = self.peek();
        if let Some(next_char) = next_char {
            if next_char != c {
                return None;
            }
            self.cursor += next_char.len_utf8();
        }
        next_char
    }

    fn peek(&self) -> Option<char> {
        self.src[self.cursor..].chars().next()
    }

    fn yield_identifier(&mut self) -> Token {
        self.advance_while(|c| c == '_' || c.is_alphanumeric());
        let identifier = self.get_slice();
        match identifier {
            "if" => Token::Keyword(Keyword::If),
            "elif" => Token::Keyword(Keyword::Elif),
            "else" => Token::Keyword(Keyword::Else),
            "for" => Token::Keyword(Keyword::For),
            "in" => Token::Keyword(Keyword::In),
            _ => Token::Identifier(identifier.to_owned()),
        }
    }

    fn yield_number(&mut self) -> Result<Token, LexerError> {
        self.advance_while(|c| c == '.' || c.is_ascii_digit());
        let slice = self.get_slice();
        let number = match slice.parse() {
            Err(err) => return Err(LexerError::NumberParseError(err)),
            Ok(number) => number,
        };
        Ok(Token::Literal(OwnedValue::Number(number)))
    }

    fn yield_string(&mut self, quote: char) -> Result<Token, LexerError> {
        // Don't include quote character in string.
        self.end_token();

        let mut string = String::new();
        loop {
            match self.expect_next_char()? {
                '\\' => {
                    let next = self.expect_next_char()?;
                    let escape = match next {
                        'n' => "\n",
                        '\\' => "\\",
                        '\n' => "",
                        _ if next == quote => &quote.to_string(),
                        _ => return Err(LexerError::UnrecognizedEscape(next)),
                    };

                    self.cursor -= 2;
                    string += self.get_slice();
                    string += escape;
                    self.cursor += 2;

                    self.end_token();
                }
                c if c == quote => {
                    self.cursor -= 1;
                    string += self.get_slice();
                    self.cursor += 1;
                    return Ok(Token::Literal(OwnedValue::String(string)));
                }
                _ => {}
            }
        }
    }

    fn advance_while(&mut self, func: fn(char) -> bool) {
        while let Some(c) = self.get_next_char() {
            if !func(c) {
                self.backup();
                break;
            }
        }
    }

    fn backup(&mut self) {
        let last_char = self.src[self.token_start_byte..self.cursor]
            .chars()
            .next_back()
            .unwrap();
        self.cursor -= last_char.len_utf8();
    }

    fn get_slice(&mut self) -> &'a str {
        &self.src[self.token_start_byte..self.cursor]
    }

    fn end_token(&mut self) {
        self.token_start_byte = self.cursor;
    }

    fn consume_whitespace(&mut self) {
        self.advance_while(char::is_whitespace);
        self.end_token();
    }
}
