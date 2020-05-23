use std;
use std::fmt::{self, Display};
use serde::{de, ser};
use serde::export::Formatter;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self where
        T: Display {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {

    fn custom<T>(msg: T) -> Self where
        T: Display {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::Syntax => f.write_str("syntax error"),
            Error::ExpectedBoolean => f.write_str("expected boolean error"),
            Error::ExpectedInteger => f.write_str("expected integer error"),
            Error::ExpectedString => f.write_str("expected string error"),
            Error::ExpectedNull => f.write_str("expected null error"),
            Error::ExpectedArray => f.write_str("expected array error"),
            Error::ExpectedArrayComma =>f.write_str("expected array comma error"),
            Error::ExpectedArrayEnd => f.write_str("expected array end error"),
            Error::ExpectedMap => f.write_str("expected map error"),
            Error::ExpectedMapColon => f.write_str("expected map colon error"),
            Error::ExpectedMapComma => f.write_str("expected map comma error"),
            Error::ExpectedMapEnd => f.write_str("expected map end error"),
            Error::ExpectedEnum => f.write_str("expected enum error"),
            Error::TrailingCharacters => f.write_str("expected trailing characters error"),
        }
    }
}

impl std::error::Error for Error {}