/// The lexer (a.k.a tokenizer) is responsible for converting the input into a one-dimensional
/// series of tokens. For example, the input `3 + (4/2)` into the lexer would yield:
/// `[Literal(3], Op(Add), OpenParen, Literal(4), Op(Divide) Literal(2), ClosingParen]`
/// This implementation is zero-allocation.
pub struct Lexer<'a> {
    /// There are many different ways of representing the input: String, &str, Vec<char>, Chars,
    /// etc. A string slice works best here because we don't have to keep converting between
    /// different types. If we need the next character, we can just use
    /// `src[cursor..].chars().next()`, which is cheap because `.chars()` returns a lazy iterator.
    src: &'a str,
    token_start_byte: usize,
    cursor: usize,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Operator(Operator),
    OpeningParen,
    ClosingParen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            token_start_byte: 0,
            cursor: 0,
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        let next_char = match self.get_next_char() {
            None => return None,
            Some(next_char) => next_char,
        };

        let token = match next_char {
            '(' => Token::OpeningParen,
            ')' => Token::ClosingParen,
            '+' => Token::Operator(Operator::Add),
            '-' => Token::Operator(Operator::Subtract),
            '*' => Token::Operator(Operator::Multiply),
            '/' => Token::Operator(Operator::Divide),
            '0'..='9' | '.' => self.yield_number(),
            c if c.is_whitespace() => {
                self.consume_whitespace();
                return self.next();
            }
            c => panic!("Unknown character: {c:?}"),
        };
        self.end_token();
        Some(token)
    }

    fn get_next_char(&mut self) -> Option<char> {
        let next_char = self.peek();
        if let Some(next_char) = next_char {
            self.cursor += next_char.len_utf8();
        }
        next_char
    }

    fn peek(&self) -> Option<char> {
        self.src[self.cursor..].chars().next()
    }

    fn yield_number(&mut self) -> Token {
        self.advance_while(|c| c == '.' || c.is_ascii_digit());
        let number = self.get_slice();
        Token::Number(number.parse().unwrap())
    }

    fn advance_while<F: Fn(char) -> bool>(&mut self, func: F) {
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
        self.advance_while(|c| c.is_whitespace());
        self.end_token();
    }
}
