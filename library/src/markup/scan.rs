use crate::{FroggiError, ScanError};

use std::collections::VecDeque;

pub fn lex(data: &str) -> Result<Vec<Token<'_>>, FroggiError> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(data);

    #[allow(irrefutable_let_patterns)]
    while let token = scanner.next_token()? {
        if token.kind == TokenKind::End {
            break;
        }
        tokens.push(token);
    }

    Ok(tokens)
}

fn is_control_character(c: u8) -> bool {
    c == b'{'
        || c == b'}'
        || c == b'('
        || c == b')'
        || c == b':'
        || c == b'&'
        || c.is_ascii_control()
        || c.is_ascii_whitespace()
}

#[derive(Debug)]
pub struct Scanner<'a> {
    start: usize,
    current: usize,
    line: usize,
    tokens: VecDeque<Token<'a>>,
    source: &'a [u8],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TokenKind {
    Text,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Ampersand,
    Identifier,
    End,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    line: usize,
    lexeme: &'a str,
}

impl Token<'_> {
    pub(crate) fn new(kind: TokenKind, line: usize, lexeme: &str) -> Token<'_> {
        Token { kind, line, lexeme }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn clone_lexeme(&self) -> String {
        self.lexeme.into()
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme
    }
}

impl<'a> Scanner<'a> {
    pub fn new(s: &str) -> Scanner<'_> {
        Scanner {
            start: 0,
            current: 0,
            line: 1,
            tokens: VecDeque::with_capacity(2),
            source: s.as_bytes(),
        }
    }

    pub fn peek_token(&mut self, idx: usize) -> Result<Token<'a>, FroggiError> {
        if self.tokens.is_empty() {
            self.next()?;
        }

        while self.tokens.len() <= idx {
            self.next()?;
        }

        Ok(self.tokens[idx])
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, FroggiError> {
        if self.tokens.is_empty() {
            self.next()?;
        }

        Ok(self.tokens.pop_front().unwrap())
    }

    fn next(&mut self) -> Result<Token<'a>, FroggiError> {
        let token = if self.at_end() {
            Token::new(TokenKind::End, self.line, "")
        } else {
            self.slurp_whitespace();
            self.start = self.current;
            Token::new(
                match self.advance() {
                    b'\0' => {
                        self.tokens
                            .push_back(Token::new(TokenKind::End, self.line, ""));
                        return Ok(Token::new(TokenKind::End, self.line, ""));
                    }

                    b'"' => self.text(),
                    b'(' => Ok(TokenKind::LeftParen),
                    b')' => Ok(TokenKind::RightParen),
                    b'{' => Ok(TokenKind::LeftBrace),
                    b'}' => Ok(TokenKind::RightBrace),

                    b'&' => Ok(TokenKind::Ampersand),
                    b':' => Ok(TokenKind::Colon),

                    _ => self.identifier(),
                }?,
                self.line,
                self.lexeme()?,
            )
        };

        self.tokens.push_back(token);
        Ok(token)
    }

    fn identifier(&mut self) -> Result<TokenKind, FroggiError> {
        while !is_control_character(self.peek()) {
            self.advance();
        }
        Ok(TokenKind::Identifier)
    }

    fn text(&mut self) -> Result<TokenKind, FroggiError> {
        let start_line = self.line;

        while !self.at_end() && self.peek() != b'"' {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            if self.peek() == b'\\' {
                self.advance();
                if self.peek() != b'\"' {
                    return self.error(ScanError::UnknownEscapeCode {
                        code: self.peek() as char,
                    });
                }
            }
            self.advance();
        }

        if self.at_end() {
            self.error(ScanError::UnterminatedString { start_line })
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

    fn lexeme(&self) -> Result<&'a str, FroggiError> {
        Ok(std::str::from_utf8(&self.source[self.start..self.current])?)
    }

    fn error(&self, error: ScanError) -> Result<TokenKind, FroggiError> {
        Err(FroggiError::scan(error, self.line))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn next_then_peek() {
        let mut s = Scanner::new("({:&name\"text\"})");
        assert_eq!(s.peek_token(0).unwrap().kind(), TokenKind::LeftParen);
        assert_eq!(s.next_token().unwrap().kind(), TokenKind::LeftParen);
        assert_eq!(s.peek_token(0).unwrap().kind(), TokenKind::LeftBrace);
        assert_eq!(s.next_token().unwrap().kind(), TokenKind::LeftBrace);
    }
}
