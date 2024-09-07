use std::num::ParseFloatError;

use thiserror::Error;

use crate::lexer::Token;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unrecognized character: {0:?}")]
    UnrecognizedCharacter(char),

    #[error("{0}")]
    NumberParseError(ParseFloatError),

    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("Unrecognized escape: \\{0}")]
    UnrecognizedEscape(char),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Lexer error: {0}")]
    LexerError(LexerError),

    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("Unexpected token: {0:?} while parsing {1:?}")]
    UnexpectedToken(Token, &'static str),
}

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("{0}")]
    StringOpError(String),

    #[error("Undefined variable: {0:?}")]
    UndefinedVariable(String),
}
