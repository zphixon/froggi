use std::io;

pub mod markup;
pub mod request;
pub mod response;

pub const FROGGI_VERSION: u8 = 0;

pub fn serialize_to_bytes(bytes: usize) -> (u8, u8) {
    assert!(bytes <= 25564);

    let high = (bytes >> 8) as u8;
    let low = (bytes & 0xff) as u8;

    (low, high)
}

pub fn serialize_to_four_bytes(bytes: usize) -> [u8; 4] {
    let a: u8 = ((bytes & 0xff_00_00_00) >> 24) as u8;
    let b: u8 = ((bytes & 0x00_ff_00_00) >> 16) as u8;
    let c: u8 = ((bytes & 0x00_00_ff_00) >> 8) as u8;
    let d: u8 = bytes as u8;

    [d, c, b, a]
}

pub fn deserialize_bytes(low: u8, high: u8) -> usize {
    ((high as usize) << 8) | (low as usize)
}

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
    RequestFormatError,
    IOError {
        error: io::Error,
    },
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
    pub fn new(error: ErrorKind) -> FroggiError {
        FroggiError { error, msg: None }
    }

    pub fn scan(error: ScanError, line: usize, file: String) -> FroggiError {
        FroggiError {
            error: ErrorKind::ScanError { error, line, file },
            msg: None,
        }
    }

    pub fn io(error: io::Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::IOError { error },
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
            msg: Some(String::from("could not decode text from utf8 to &str")),
        }
    }
}

impl From<std::string::FromUtf8Error> for FroggiError {
    fn from(_: std::string::FromUtf8Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::EncodingError,
            msg: Some(String::from("could not decode text from utf8 to String")),
        }
    }
}

impl From<io::Error> for FroggiError {
    fn from(error: io::Error) -> FroggiError {
        FroggiError::io(error)
    }
}

pub fn hello() {
    println!("ribbit!");
}
