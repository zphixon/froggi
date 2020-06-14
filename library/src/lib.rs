pub mod ast;
pub mod color;
pub mod font;
pub mod parse;
pub mod style;

use ast::*;
use color::*;
use font::*;
use parse::*;
use style::*;

#[derive(Debug)]
pub enum ScanError {
    UnknownStyle,
    UnknownItem,
    UnknownFontStyle,
    InvalidColor,
    UnknownEscapeCode,
    UnterminatedString,
    Utf8Error,
}

#[derive(Debug)]
pub enum ErrorKind {
    EncodingError,
    ScanError {
        error: ScanError,
        line: usize,
        file: String,
    },
}

#[derive(Debug)]
pub struct FroggiError {
    error: ErrorKind,
    msg: Option<String>,
}

impl FroggiError {
    pub fn scan(error: ScanError, line: usize, file: String) -> FroggiError {
        FroggiError {
            error: ErrorKind::ScanError { error, line, file },
            msg: None,
        }
    }
}

impl AddMsg for FroggiError {
    fn msg(self, msg: String) -> FroggiError {
        FroggiError {
            msg: Some(msg),
            ..self
        }
    }

    fn msg_str(self, msg: &str) -> FroggiError {
        FroggiError {
            msg: Some(msg.to_owned()),
            ..self
        }
    }
}

trait AddMsg {
    fn msg(self, msg: String) -> Self;
    fn msg_str(self, msg: &str) -> Self;
}

impl<T> AddMsg for Result<T, FroggiError> {
    fn msg(self, msg: String) -> Self {
        self.map_err(|e| e.msg(msg))
    }

    fn msg_str(self, msg: &str) -> Self {
        self.map_err(|e| e.msg_str(msg))
    }
}

impl From<std::str::Utf8Error> for FroggiError {
    fn from(_: std::str::Utf8Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::EncodingError,
            msg: Some(String::from("could not decode text from utf8")),
        }
    }
}

pub fn hello() {
    println!("ribbit!");
}
