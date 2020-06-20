pub mod ast;
pub mod parse;
pub mod scan;

use crate::{FroggiError, ScanError};
use scan::Token;

#[derive(Debug)]
pub struct Page<'a> {
    pub page_styles: Vec<PageStyle<'a>>,
    pub items: Vec<PageItem<'a>>,
}

#[derive(Debug)]
pub enum PageStyleSelector<'a> {
    UserDefined { name: Token<'a> },
    Builtin { name: Token<'a> },
}

#[derive(Debug)]
pub struct PageStyle<'a> {
    selector: PageStyleSelector<'a>,
    styles: Vec<InlineStyle<'a>>,
}

#[derive(Debug)]
pub struct PageItem<'a> {
    // a None value implies a :text item
    pub builtin: Option<Token<'a>>,
    pub defined_styles: Vec<Token<'a>>,
    pub inline_styles: Vec<InlineStyle<'a>>,
    pub payload: ItemPayload<'a>,
}

#[derive(Debug)]
pub enum ItemPayload<'a> {
    Text { text: Vec<Token<'a>> },
    Children { children: Vec<PageItem<'a>> },
}

#[derive(Debug)]
pub enum InlineStyle<'a> {
    NoArgs { name: Token<'a> },
    Arg { name: Token<'a>, arg: Token<'a> },
}

pub fn escape_string(t: &Token) -> Result<String, FroggiError> {
    match t.kind() {
        scan::TokenKind::Text => {
            let s = t.lexeme();
            let mut value = Vec::new();
            let line_start = t.line();
            let mut line = line_start;

            let mut i = 1;
            while i < s.as_bytes().len() - 1 {
                let byte = s.as_bytes()[i];
                if byte == b'\n' {
                    line += 1;
                }

                if byte == b'\\' {
                    i += 1;
                    let byte = s.as_bytes()[i];
                    match byte {
                        b'n' => {
                            value.push(b'\n');
                        }
                        b'r' => {
                            value.push(b'\r');
                        }
                        b'\\' => {
                            value.push(b'\\');
                        }
                        b'"' => {
                            value.push(b'"');
                        }
                        b't' => {
                            value.push(b'\t');
                        }
                        b'\n' => {
                            while i < s.as_bytes().len() - 1
                                && s.as_bytes()[i].is_ascii_whitespace()
                            {
                                i += 1;
                            }
                            i -= 1;
                        }
                        c => {
                            return Err(FroggiError::scan(
                                ScanError::UnknownEscapeCode { code: c as char },
                                line,
                            ));
                        }
                    }
                } else {
                    value.push(byte);
                }

                i += 1;
            }

            Ok(String::from_utf8(value)?)
        }
        _ => {
            panic!("Cannot escape string from token {:?}", t);
        }
    }
}
