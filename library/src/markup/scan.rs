//! Scan a byte sequence into a froggi markup page. Zero-copy.

use crate::markup::document::Direction;
use crate::Result;
use crate::{Encoding, UnbalancedParentheses, UnknownEscapeCode, UnterminatedString};

use snafu::ResultExt;

fn is_control_character(c: u8) -> bool {
    c == b'{'
        || c == b'}'
        || c == b'('
        || c == b')'
        || c == b'^'
        || c == b'#'
        || c == b'&'
        || c == b'"'
        || c == b'\''
        || c == b';'
        || c.is_ascii_control()
        || c.is_ascii_whitespace()
}

/// Kinds of token.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TokenKind {
    /// String, in single or double quotes
    String,
    /// Left paren
    LeftParen,
    /// Right paren
    RightParen,
    /// Left brace
    LeftBrace,
    /// Right brace
    RightBrace,

    /// & item reference
    Blob,
    /// ^ link
    Link,
    /// \# anchor
    Anchor,
    /// inline - inline layout type
    Inline,
    /// wide - horizontal layout type
    Wide,
    /// tall - vertical layout type
    Tall,
    /// text - text type
    Text,

    /// Monospace font
    Mono,
    /// Serif font
    Serif,
    /// Sans-serif font
    Sans,
    /// Bold font
    Bold,
    /// Italic font
    Italic,
    /// Underlined text
    Underline,
    /// Strike-through text
    Strike,

    /// Foreground text color, takes an argument string in hexadecimal
    Fg,
    /// Background color, takes an argument string in hexadecimal
    Bg,
    /// Fill, takes a floating-point argument string
    Fill,
    /// Text size, takes an integer argument string
    Size,

    /// A user-defined style
    Identifier,

    /// The end of the page
    End,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::String => "string",
                TokenKind::LeftParen => "(",
                TokenKind::RightParen => ")",
                TokenKind::LeftBrace => "{",
                TokenKind::RightBrace => "}",
                TokenKind::Blob => "&",
                TokenKind::Link => "^",
                TokenKind::Anchor => "#",
                TokenKind::Inline => "inline",
                TokenKind::Wide => "wide",
                TokenKind::Tall => "tall",
                TokenKind::Text => "text",
                TokenKind::Mono => "mono",
                TokenKind::Serif => "serif",
                TokenKind::Sans => "sans",
                TokenKind::Bold => "bold",
                TokenKind::Italic => "italic",
                TokenKind::Underline => "underline",
                TokenKind::Strike => "strike",
                TokenKind::Fg => "fg",
                TokenKind::Bg => "bg",
                TokenKind::Fill => "fill",
                TokenKind::Size => "size",
                TokenKind::Identifier => "identifier",
                TokenKind::End => "EOF",
            }
        )
    }
}

/// A token.
#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    line: usize,
    lexeme: &'a str,
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexeme == other.lexeme
    }
}

impl Eq for Token<'_> {}

impl std::hash::Hash for Token<'_> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.kind.hash(hasher);
        self.lexeme.hash(hasher);
    }
}

impl Token<'_> {
    pub(crate) fn new(kind: TokenKind, line: usize, lexeme: &str) -> Token<'_> {
        Token { kind, line, lexeme }
    }

    /// Get the kind of the token
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Get the line number of the token
    pub fn line(&self) -> usize {
        self.line
    }

    /// Clone the lexeme of the token
    pub fn clone_lexeme(&self) -> String {
        self.lexeme().into()
    }

    /// Get the lexeme of the token
    pub fn lexeme(&self) -> &str {
        match self.kind {
            TokenKind::String => &self.lexeme[1..self.lexeme.len() - 1],
            _ => self.lexeme,
        }
    }

    pub fn may_be_styled(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Identifier
                | TokenKind::Link
                | TokenKind::Wide
                | TokenKind::Tall
                | TokenKind::Text
        )
    }

    pub fn direction(&self) -> Direction {
        match self.kind {
            TokenKind::Wide => Direction::Horizontal,
            TokenKind::Inline => Direction::Inline,
            _ => Direction::Vertical,
        }
    }
}

/// Struct used to scan a froggi markup page.
#[derive(Debug)]
pub struct Scanner<'a> {
    start: usize,
    current: usize,
    line: usize,
    paren_level: usize,
    token: Option<Token<'a>>,
    source: &'a [u8],
}

impl<'a> Scanner<'a> {
    /// Construct a new scanner
    pub fn new(s: &str) -> Scanner<'_> {
        Scanner {
            start: 0,
            current: 0,
            line: 1,
            paren_level: 0,
            token: None,
            source: s.as_bytes(),
        }
    }

    /// True if the scanner is not currently inside a page expression
    pub fn at_top_level(&self) -> bool {
        self.paren_level == 0
    }

    /// Peek a token in the token stream
    pub fn peek_token(&mut self) -> Result<Token<'a>> {
        if self.token.is_none() {
            self.next()?;
        }

        Ok(self.token.unwrap())
    }

    /// Get the next token in the stream
    pub fn next_token(&mut self) -> Result<Token<'a>> {
        if self.token.is_none() {
            self.next()?;
        }

        Ok(self.token.take().unwrap())
    }

    fn next(&mut self) -> Result<Token<'a>> {
        let token = if self.at_end() {
            Token::new(TokenKind::End, self.line, "")
        } else {
            self.slurp_whitespace();
            self.start = self.current;
            Token::new(
                match self.advance() {
                    b'\0' => {
                        self.token = Some(Token::new(TokenKind::End, self.line, ""));
                        return Ok(Token::new(TokenKind::End, self.line, ""));
                    }

                    b'"' | b'\'' => self.text(),
                    b'{' => Ok(TokenKind::LeftBrace),
                    b'}' => Ok(TokenKind::RightBrace),

                    b'(' => {
                        self.paren_level += 1;
                        Ok(TokenKind::LeftParen)
                    }

                    b')' => {
                        if self.paren_level != 0 {
                            self.paren_level -= 1;
                            Ok(TokenKind::RightParen)
                        } else {
                            UnbalancedParentheses { line: self.line }.fail()
                            //Err(FroggiError::parse(
                            //    crate::ParseError::UnbalancedParentheses,
                            //    self.line,
                            //))
                        }
                    }

                    b'&' => Ok(TokenKind::Blob),
                    b'^' => Ok(TokenKind::Link),
                    b'#' => Ok(TokenKind::Anchor),

                    _ => self.identifier(),
                }?,
                self.line,
                self.lexeme()?,
            )
        };

        self.token = Some(token);
        Ok(token)
    }

    fn identifier(&mut self) -> Result<TokenKind> {
        while !is_control_character(self.peek()) {
            self.advance();
        }

        match self.lexeme()? {
            "wide" => Ok(TokenKind::Wide),
            "tall" => Ok(TokenKind::Tall),
            "inline" => Ok(TokenKind::Inline),
            "text" => Ok(TokenKind::Text),
            "mono" => Ok(TokenKind::Mono),
            "serif" => Ok(TokenKind::Serif),
            "sans" => Ok(TokenKind::Sans),
            "bold" => Ok(TokenKind::Bold),
            "italic" => Ok(TokenKind::Italic),
            "underline" => Ok(TokenKind::Underline),
            "strike" => Ok(TokenKind::Strike),
            "fg" => Ok(TokenKind::Fg),
            "bg" => Ok(TokenKind::Bg),
            "fill" => Ok(TokenKind::Fill),
            "size" => Ok(TokenKind::Size),
            _ => Ok(TokenKind::Identifier),
        }
    }

    fn text(&mut self) -> Result<TokenKind> {
        let start_line = self.line;

        while !self.at_end() && (self.peek() != b'\'' && self.peek() != b'"') {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            if self.peek() == b'\\' {
                self.advance();
                if self.peek() != b'\'' || self.peek() != b'"' {

                    //return self.error(ScanError::UnknownEscapeCode {
                    //    code: self.peek() as char,
                    //});
                }
            }
            self.advance();
        }

        if self.at_end() {
            UnterminatedString { line: start_line }.fail()
            //self.error(ScanError::UnterminatedString { start_line })
        } else {
            self.advance();
            Ok(TokenKind::String)
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

    fn lexeme(&self) -> Result<&'a str> {
        Ok(std::str::from_utf8(&self.source[self.start..self.current]).context(Encoding)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn next_then_peek() {
        let mut s = Scanner::new("({:&name\'text\'})");
        assert_eq!(s.peek_token().unwrap().kind(), TokenKind::LeftParen);
        assert_eq!(s.next_token().unwrap().kind(), TokenKind::LeftParen);
        assert_eq!(s.peek_token().unwrap().kind(), TokenKind::LeftBrace);
        assert_eq!(s.next_token().unwrap().kind(), TokenKind::LeftBrace);
    }

    #[test]
    fn smooshed_tokens() {
        let mut s = Scanner::new(r#"a(&&#&text'hello'^h^h^'text'something{"#);
        let mut tokens = Vec::new();
        while let Ok(token) = s.next_token() {
            if token.kind() == TokenKind::End {
                break;
            }
            tokens.push(token);
        }
        dbg!(&tokens);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, 1, "a"),
                Token::new(TokenKind::LeftParen, 1, "("),
                Token::new(TokenKind::Blob, 1, "&"),
                Token::new(TokenKind::Blob, 1, "&"),
                Token::new(TokenKind::Anchor, 1, "#"),
                Token::new(TokenKind::Blob, 1, "&"),
                Token::new(TokenKind::Text, 1, "text"),
                Token::new(TokenKind::String, 1, "\'hello\'"),
                Token::new(TokenKind::Link, 1, "^"),
                Token::new(TokenKind::Identifier, 1, "h"),
                Token::new(TokenKind::Link, 1, "^"),
                Token::new(TokenKind::Identifier, 1, "h"),
                Token::new(TokenKind::Link, 1, "^"),
                Token::new(TokenKind::String, 1, "\'text\'"),
                Token::new(TokenKind::Identifier, 1, "something"),
                Token::new(TokenKind::LeftBrace, 1, "{"),
            ]
        );
    }

    #[test]
    fn comments() {
        let mut scanner = Scanner::new("(ident;\ni");

        let mut tokens = Vec::new();
        while let Ok(token) = scanner.next_token() {
            if token.kind() == TokenKind::End {
                break;
            }
            tokens.push(token);
        }

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::LeftParen, 1, "("),
                Token::new(TokenKind::Identifier, 1, "ident"),
                Token::new(TokenKind::Identifier, 2, "i"),
            ]
        );
    }

    #[test]
    fn line_number_partial_eq() {
        assert_eq!(
            Token::new(TokenKind::Identifier, 1, "i"),
            Token::new(TokenKind::Identifier, 25565, "i")
        );

        assert_eq!(
            Token::new(TokenKind::LeftParen, 1, "("),
            Token::new(TokenKind::LeftParen, 25565, "(")
        );
    }

    #[test]
    fn hash_token() {
        let mut map = std::collections::HashMap::new();
        map.insert(Token::new(TokenKind::Identifier, 1, "i"), ());
        assert!(map.contains_key(&Token::new(TokenKind::Identifier, 3889583, "i")));
    }
}
