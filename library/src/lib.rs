//! Froggi protocol library
//!
//! Add `features = ['markup']` to get markup AST and parsing, and for more layout-oriented types.

use std::fmt;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::str;

use snafu::{ResultExt, Snafu};

pub use uuid::Uuid;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "markup")]
pub mod markup;

pub mod protocol;
pub mod request;
pub mod response;

use markup::scan::TokenKind;
use request::RequestKind;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Froggi version.
pub const FROGGI_VERSION: u8 = 0;

/// Send a froggi request to a server and return its response.
pub fn send_request(
    to: impl ToSocketAddrs,
    request: &str,
    kind: RequestKind,
) -> Result<response::Response> {
    let mut stream = TcpStream::connect(to).context(TcpConnection)?;
    stream
        .write_all(&request::Request::new_with_id(request, Uuid::nil(), kind)?.into_bytes())
        .context(IO)?;

    Ok(response::Response::from_bytes(&mut stream)?)
}

/// Send a froggi request to a server with a client ID and return its response.
pub fn send_request_with_id(
    to: impl ToSocketAddrs,
    request: &str,
    id: Uuid,
    kind: RequestKind,
) -> Result<response::Response> {
    let mut stream = TcpStream::connect(to).context(TcpConnection)?;
    stream
        .write_all(&request::Request::new_with_id(request, id, kind)?.into_bytes())
        .context(IO)?;

    Ok(response::Response::from_bytes(&mut stream)?)
}

#[cfg(feature = "markup")]
pub fn response_from_file(
    path: impl AsRef<std::path::Path>,
) -> Result<response::Response, Vec<Error>> {
    // TODO this is kind of garbage

    let path = path.as_ref();
    let data = std::fs::read_to_string(&path).unwrap(); //.context(ReadFile { filename: path.to_path_buf() })?;
                                                        //.map_err(|error| FroggiError::new(ErrorKind::IOError { error }))
                                                        //.msg(format!("could not read '{}'", path.display()))?;

    let page = markup::parse::parse(&data)?;

    let item_names = page.item_names();

    let mut item_data = Vec::new();
    for name in item_names.iter() {
        item_data.push(
            std::fs::read(path.parent().unwrap().join(name)).unwrap()
            //.map_err(|error| {
            //    FroggiError::new(ErrorKind::IOError { error })
            //        .msg(format!("could not read item '{}'", name))
            //})?,
        );
    }

    let items = item_names
        .into_iter()
        .zip(item_data.into_iter())
        .map(|(name, data)| {
            // TODO item kind
            response::Item::new(name, response::ItemKind::Png, data)
        })
        .collect();

    Ok(response::Response::new(
        response::ResponseKind::Page,
        data,
        items,
    )?)
}

/// Serialize a usize into a little-endian pair of bytes.
pub fn serialize_to_bytes(bytes: usize) -> Result<(u8, u8)> {
    if bytes > u16::MAX as usize {
        Err(Error::BitWidth {
            wanted: 16,
            got: bytes,
        })?
        //return Err(FroggiError::new(ErrorKind::BitWidthError {
        //    wanted: 16,
        //    got: bytes,
        //}));
    }

    let high = (bytes >> 8) as u8;
    let low = (bytes & 0xff) as u8;

    Ok((low, high))
}

/// Serialize a usize into a little-endian quartet of bytes.
pub fn serialize_to_four_bytes(bytes: usize) -> Result<[u8; 4]> {
    if bytes > u32::MAX as usize {
        Err(Error::BitWidth {
            wanted: 32,
            got: bytes as usize,
        })?
        //return Err(FroggiError::new(ErrorKind::BitWidthError {
        //    wanted: 32,
        //    got: bytes,
        //}));
    }

    let a: u8 = ((bytes & 0xff_00_00_00) >> 24) as u8;
    let b: u8 = ((bytes & 0x00_ff_00_00) >> 16) as u8;
    let c: u8 = ((bytes & 0x00_00_ff_00) >> 8) as u8;
    let d: u8 = bytes as u8;

    Ok([d, c, b, a])
}

/// Deserialize a pair of bytes into a usize.
pub fn deserialize_bytes(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 2 {
        Err(Error::ArrayLength {
            wanted: 2,
            got: bytes.len(),
        })?
    }

    let low = bytes[0];
    let high = bytes[1];
    Ok(((high as usize) << 8) | (low as usize))
}

/// Deserialize a quartet of bytes into a usize.
pub fn deserialize_four_bytes(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 4 {
        Err(Error::ArrayLength {
            wanted: 4,
            got: bytes.len(),
        })?
    }

    Ok(((bytes[3] as usize) << 24)
        | ((bytes[2] as usize) << 16)
        | ((bytes[1] as usize) << 8)
        | (bytes[0] as usize))
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("line {} - unknown escape code {}", line, code))]
    UnknownEscapeCode { line: usize, code: char },
    #[snafu(display("unterminated string starting on line {}", line))]
    UnterminatedString { line: usize },
    #[snafu(display(
        "line {} - unexpected token: got '{}', expected {}",
        line,
        expected,
        got
    ))]
    UnexpectedToken {
        line: usize,
        expected: crate::markup::scan::TokenKind,
        got: String,
    },
    #[snafu(display("line {} - unbalanced parentheses", line))]
    UnbalancedParentheses { line: usize },
    #[snafu(display("line {} - expected style (fg, bold, etc) got '{}'", line, got))]
    ExpectedStyle { line: usize, got: String },
    #[snafu(display("line {} - expected expression, got '{}'", line, got))]
    ExpectedExpression { line: usize, got: String },
    #[snafu(display("line {} - unknown style '{}'", line, style))]
    UnknownStyle { line: usize, style: String },
    #[snafu(display("line {} - recursive style '{}', currently not allowed", line, style))]
    RecursiveStyle { line: usize, style: String },
    #[snafu(display(
        "line {} - incorrect number format, wanted {}, got '{}'",
        line,
        wanted,
        num
    ))]
    IncorrectFloatFormat {
        source: std::num::ParseFloatError,
        line: usize,
        num: String,
        wanted: String,
    },
    #[snafu(display(
        "line {} - incorrect number format, wanted {}, got '{}'",
        line,
        wanted,
        num
    ))]
    IncorrectHexFormat {
        source: hex::FromHexError,
        line: usize,
        num: String,
        wanted: String,
    },
    #[snafu(display(
        "line {} - incorrect number format, wanted {}, got '{}'",
        line,
        wanted,
        num
    ))]
    IncorrectIntFormat {
        source: std::num::ParseIntError,
        line: usize,
        num: String,
        wanted: String,
    },
    #[snafu(display(
        "line {} - incorrect number format, wanted {}, got '{}'",
        line,
        wanted,
        num
    ))]
    IncorrectNumberFormat {
        line: usize,
        num: String,
        wanted: String,
    },
    #[snafu(display("array length incorrect - wanted {}, got {}", wanted, got))]
    ArrayLength { wanted: usize, got: usize },
    #[snafu(display("bit width incorrect - wanted {}, got {}", wanted, got))]
    BitWidth { wanted: usize, got: usize },
    #[snafu(display("utf8 error - {}", source))]
    Encoding { source: std::str::Utf8Error },
    #[snafu(display("utf8 error - {}", source))]
    FromEncoding { source: std::string::FromUtf8Error },
    #[snafu(display("request was formatted incorrectly"))]
    RequestFormat,
    #[snafu(display("response was formatted incorrectly"))]
    ResponseFormat,
    #[snafu(display("io error - {}", source))]
    IO { source: std::io::Error },
    #[snafu(display("could not read file '{}' - {}", filename.display(), source))]
    ReadFile {
        source: std::io::Error,
        filename: std::path::PathBuf,
    },
    #[snafu(display("could not connect to server - {}", source))]
    TcpConnection { source: std::io::Error },
}

impl From<Error> for Vec<Error> {
    fn from(error: Error) -> Vec<Error> {
        vec![error]
    }
}

/*
/// FML document scan error.
#[derive(Debug)]
pub enum ScanError {
    /// Escape code in a string is unknown
    UnknownEscapeCode {
        /// The unknown escape code character
        code: char,
    },
    /// A string is unterminated
    UnterminatedString {
        /// The line the string starts on
        start_line: usize,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanError::UnknownEscapeCode { code } => write!(f, "unknown escape code: {}", code),
            ScanError::UnterminatedString { start_line } => {
                write!(f, "unterminated string starting on line {}", start_line)
            }
        }
    }
}

/// FML document parse error.
#[derive(Debug)]
pub enum ParseError {
    /// An unexpected token was encountered
    UnexpectedToken {
        /// The token that was expected
        expected: TokenKind,
        /// The token that was received
        got: String,
    },
    /// A parenthesis is missing somewhere
    UnbalancedParentheses,
    /// A non-style token was encountered
    ExpectedStyle {
        /// The token that was encountered
        got: String,
    },
    /// An expression was expected in the document root
    ExpectedExpression {
        /// We got this instead
        got: String,
    },
    /// The style was unrecognized, either not built-in, undefined, or misspelled
    UnknownStyle {
        /// The style that was found
        style: String,
    },
    /// We don't allow recursive style definitions currently, this may change
    RecursiveStyle {
        /// The style that was recursive
        style: String,
    },
    /// The number was formatted incorrectly
    IncorrectNumberFormat {
        /// Fake number
        num: String,
        /// Real number
        wanted: String,
    },
}

#[rustfmt::skip]
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, got }
                => write!(f, "unexpected token: expected {:?}, got {}", expected, got),
            ParseError::UnbalancedParentheses
                => write!(f, "unbalanced parentheses"),
            ParseError::ExpectedStyle { got }
                => write!(f, "expected style, got {:?}", got),
            ParseError::ExpectedExpression { got }
                => write!(f, "expected page expression or page style, got {:?}", got),
            ParseError::UnknownStyle { style }
                => write!(f, "unknown style {:?}", style),
            ParseError::RecursiveStyle { style }
                => write!(f, "unknown style {:?}", style),
            ParseError::IncorrectNumberFormat { num, wanted }
                => write!(f, "incorrect number format: wanted {}, {:?}", wanted, num),
        }
    }
}

/// Errors that are possible in the froggi protocol.
#[derive(Debug)]
pub enum ErrorKind {
    /// The array was too small or large
    ArrayLengthError {
        /// Size we wanted
        wanted: usize,
        /// Size we got
        got: usize,
    },
    /// The integer wasn't able to fit in the specified bit width
    BitWidthError {
        /// Number of bits we wanted
        wanted: usize,
        /// The value we got instead
        got: usize,
    },
    /// The string was not UTF-8
    EncodingError {
        /// Stdlib Utf8Error that caused this
        error: str::Utf8Error,
    },
    /// The request was formatted incorrectly
    RequestFormatError,
    /// The response was formatted incorrectly
    ResponseFormatError,
    /// Encountered a problem in reading or writing
    IOError {
        /// The error that occurred
        error: io::Error,
    },
    /// Couldn't scan the document
    ScanError {
        /// The scan error
        error: ScanError,
        /// Where it happened
        line: usize,
    },
    /// Couldn't parse the document
    ParseError {
        /// The parse error
        error: ParseError,
        /// Where it happened
        line: usize,
    },
}

#[rustfmt::skip]
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ArrayLengthError { wanted, got }
                => write!(f, "array length - wanted {} elements, got {}", wanted, got),
            ErrorKind::BitWidthError { wanted, got }
                => write!(f, "bit width error - value {} cannot fit in {} bits", got, wanted),
            ErrorKind::EncodingError { error }
                => write!(f, "encoding error - {}", error),
            ErrorKind::RequestFormatError
                => write!(f, "request format error - {:?}", self),
            ErrorKind::ResponseFormatError
                => write!(f, "response format error - {:?}", self),
            ErrorKind::IOError { error }
                => write!(f, "io error - {}", error),
            ErrorKind::ScanError { error, line }
                => write!(f, "scan error on line {} - {}", line, error),
            ErrorKind::ParseError { error, line }
                => write!(f, "parse error on line {} - {}", line, error),
        }
    }
}

/// A froggi protocol library error.
#[derive(Debug)]
pub struct FroggiError {
    error: ErrorKind,
    msg: Option<String>,
}

impl FroggiError {
    /// Create a new error
    pub fn new(error: ErrorKind) -> FroggiError {
        FroggiError { error, msg: None }
    }

    /// Get the kind of error
    pub fn kind(&self) -> &ErrorKind {
        &self.error
    }

    /// Create a new scan error
    pub fn scan(error: ScanError, line: usize) -> FroggiError {
        FroggiError {
            error: ErrorKind::ScanError { error, line },
            msg: None,
        }
    }

    /// Create a new IO error
    pub fn io(error: io::Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::IOError { error },
            msg: None,
        }
    }

    /// Create a new parse error
    pub fn parse(error: ParseError, line: usize) -> FroggiError {
        FroggiError {
            error: ErrorKind::ParseError { error, line },
            msg: None,
        }
    }
}

impl AddMsg for FroggiError {
    fn msg(mut self, msg: String) -> FroggiError {
        match self.msg {
            Some(ref mut message) => {
                message.push_str(&format!(", {}", msg));
            }
            None => self.msg = Some(msg),
        }

        FroggiError { ..self }
    }

    fn msg_str(self, msg: &str) -> FroggiError {
        self.msg(String::from(msg))
    }
}

impl std::error::Error for FroggiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.error {
            ErrorKind::ArrayLengthError { .. } => None,
            ErrorKind::BitWidthError { .. } => None,
            ErrorKind::EncodingError { error } => error.source(),
            ErrorKind::RequestFormatError => None,
            ErrorKind::ResponseFormatError => None,
            ErrorKind::IOError { error } => error.source(),
            ErrorKind::ScanError { .. } => None,
            ErrorKind::ParseError { .. } => None,
        }
    }
}

impl fmt::Display for FroggiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.error,
            if self.msg.is_some() { " (" } else { "" },
            self.msg.clone().unwrap_or(String::new()),
            if self.msg.is_some() { ")" } else { "" },
        )
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

impl From<FroggiError> for Vec<FroggiError> {
    fn from(error: FroggiError) -> Vec<FroggiError> {
        vec![error]
    }
}

impl From<str::Utf8Error> for FroggiError {
    fn from(error: str::Utf8Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::EncodingError { error },
            msg: Some(String::from("could not decode text from utf8 to &str")),
        }
    }
}

impl From<std::string::FromUtf8Error> for FroggiError {
    fn from(error: std::string::FromUtf8Error) -> FroggiError {
        FroggiError {
            error: ErrorKind::EncodingError {
                error: error.utf8_error(),
            },
            msg: Some(String::from("could not decode text from utf8 to String")),
        }
    }
}

impl From<io::Error> for FroggiError {
    fn from(error: io::Error) -> FroggiError {
        FroggiError::io(error)
    }
}
 */

/// Create a u8-based enum with From and Into impls.
#[macro_export]
macro_rules! u8enum {
    ($name:ident { $($variant:ident = $value:expr),* $(,)? }) => {
        #[repr(u8)]
        #[derive(Copy, Clone, Debug)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                match self {
                    $($name::$variant => $value,)*
                }
            }
        }

        impl From<u8> for $name {
            fn from(b: u8) -> $name {
                match b {
                    $($value => $name::$variant,)*
                    _ => $name::Unknown,
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[derive(Debug)]
    pub struct TestByteError {
        pub real: u8,
        pub test: u8,
        pub i: usize,
    }

    pub fn test_bytes(real: &[u8], test: &[u8]) -> Result<(), TestByteError> {
        for (i, (test, real)) in test.iter().cloned().zip(real.iter().cloned()).enumerate() {
            if test != real {
                return Err(TestByteError { real, test, i });
            }
        }

        Ok(())
    }
}
