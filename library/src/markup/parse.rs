use crate::{FroggiError, ScanError, AddMsg};
use super::*;

pub fn parse(data: &str, filename: String) -> Vec<Item<'_>> {
    let _tokens = lex(data, filename);
    vec![]
}

pub fn lex(data: &str, filename: String) -> Result<Vec<Token<'_>>, FroggiError> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(data, filename);

    #[allow(irrefutable_let_patterns)]
    while let token = scanner.next_token()? {
        if token.kind == TokenKind::End {
            break;
        }
        tokens.push(token);
    }

    Ok(tokens)
}

#[derive(Debug)]
pub struct Scanner<'a> {
    filename: String,
    start: usize,
    current: usize,
    line: usize,
    source: &'a [u8],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TokenKind {
    Text,
    LeftParen,
    RightParen,
    Builtin,         // txt
    User,            // QuoteBox
    Not,             // !
    _At,             // @
    ForegroundColor, // #
    FontChoice,      // $
    Fill,            // %
    _Caret,          // ^
    _Ampersand,      // &
    BackgroundColor, // *
    End,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    line: usize,
    lexeme: &'a str,
}

impl Token<'_> {
    fn new(kind: TokenKind, line: usize, lexeme: &str) -> Token<'_> {
        Token { kind, line, lexeme }
    }
}

impl<'a> Scanner<'a> {
    fn new(s: &str, filename: String) -> Scanner<'_> {
        Scanner {
            filename,
            start: 0,
            current: 0,
            line: 1,
            source: s.as_bytes(),
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>, FroggiError> {
        if self.at_end() {
            Ok(Token::new(TokenKind::End, self.line, ""))
        } else {
            self.slurp_whitespace();
            self.start = self.current;
            Ok(Token::new(
                match self.advance() {
                    b'\0' => return Ok(Token::new(TokenKind::End, self.line, "")),

                    b'"' => self.text(),
                    b'(' => Ok(TokenKind::LeftParen),
                    b')' => Ok(TokenKind::RightParen),

                    b'!' => Ok(TokenKind::Not),
                    b'#' => self.color(),
                    b'$' => self.font_choice(),
                    b'%' => self.fill(),
                    b'*' => self.color(),

                    b'@' | b'^' | b'&' => self
                        .error(ScanError::UnknownStyle)
                        .msg(format!("\"{}\"", self.peek() as char)),

                    c if c.is_ascii_lowercase() => {
                        self.name();
                        Ok(TokenKind::Builtin)
                    }
                    c if c.is_ascii_uppercase() => {
                        self.name();
                        Ok(TokenKind::User)
                    }

                    _ => self.error(ScanError::UnknownItem).msg(format!("\"{}\"", {
                        self.name();
                        self.lexeme()?
                    })),
                }?,
                self.line,
                self.lexeme()?,
            ))
        }
    }

    fn fill(&mut self) -> Result<TokenKind, FroggiError> {
        while !self.at_end() && self.advance().is_ascii_digit() {}

        Ok(TokenKind::Fill)
    }

    fn font_choice(&mut self) -> Result<TokenKind, FroggiError> {
        self.name();
        match &self.lexeme()?[1..] {
            "serif" | "sans" | "mono" | "italic" | "bold" | "strike" | "underline" => {
                Ok(TokenKind::FontChoice)
            }
            _ => self
                .error(ScanError::UnknownFontStyle)
                .msg(format!("\"{}\"", self.lexeme()?)),
        }
    }

    fn color(&mut self) -> Result<TokenKind, FroggiError> {
        let color_type = self.peek_back(1);
        for _ in 1..=6 {
            let c = self.advance();

            if b'a' <= c && c <= b'f' || b'A' <= c && c <= b'F' || b'0' <= c && c <= b'9' {
                continue;
            } else {
                return self
                    .error(ScanError::InvalidColor)
                    .msg(format!("\"{}\"", self.lexeme()?));
            }
        }

        Ok(match color_type {
            b'#' => TokenKind::ForegroundColor,
            b'*' => TokenKind::BackgroundColor,
            _ => unreachable!(),
        })
    }

    fn name(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
    }

    fn text(&mut self) -> Result<TokenKind, FroggiError> {
        let line_start = self.line;

        while !self.at_end() && self.peek() != b'"' {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            if self.peek() == b'\\' {
                self.advance();
                if self.peek() != b'\"' {
                    return self.error(ScanError::UnknownEscapeCode).msg(format!(
                        "\\{} is not a valid escape sequence",
                        self.peek() as char
                    ));
                }
            }
            self.advance();
        }

        if self.at_end() {
            self.error(ScanError::UnterminatedString)
                .msg(format!("starting on line {}", line_start))
        } else {
            self.advance();
            Ok(TokenKind::Text)
        }
    }

    fn slurp_whitespace(&mut self) {
        while self.peek() == b';' || self.peek().is_ascii_whitespace() {
            if self.peek() == b';' {
                while !self.at_end() && self.peek() != b'\n' {
                    self.advance();
                }
            }

            while !self.at_end() && self.peek().is_ascii_whitespace() {
                if self.advance() == b'\n' {
                    self.line += 1;
                }
            }
        }
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source.get(self.current - 1).copied().unwrap_or(b'\0')
    }

    fn peek(&self) -> u8 {
        if self.at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_back(&self, offset: usize) -> u8 {
        self.source[self.current - offset]
    }

    fn lexeme(&self) -> Result<&'a str, FroggiError> {
        Ok(std::str::from_utf8(&self.source[self.start..self.current])?)
    }

    fn error(&self, error: ScanError) -> Result<TokenKind, FroggiError> {
        Err(FroggiError::scan(error, self.line, self.filename.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample() {
        let sample = include_str!("../../../sample_markdown.scm");
        lex(sample, "../../sample_markdown.scm".into()).unwrap();
    }
}
