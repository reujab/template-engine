use std::num::ParseFloatError;

use thiserror::Error;

use crate::{lexer::Token, value::Value};

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unrecognized character: {0:?}")]
    UnexpectedCharacter(char),

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

    // Sometimes we expect an unexpected token. (See `parse_if` and `parse_for`.)
    #[error("Unexpected token: {0:?}")]
    ExpectedToken(Token),
}

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("{0}")]
    OperationError(String),

    #[error("Undefined variable: {0:?}")]
    UndefinedVariable(String),

    #[error("Cannot iterate over {0:?}")]
    IterateError(Value),
}
