use crate::{FroggiError, ScanError};

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TokenKind {
    String,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Blob,
    Link,
    Anchor,
    Inline,
    Box,
    VBox,
    Text,
    ImplicitText,

    Mono,
    Serif,
    Sans,
    Bold,
    Italic,
    Underline,
    Strike,

    Fg,
    Bg,
    Fill,
    Size,

    Identifier,
    End,
}

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

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn clone_lexeme(&self) -> String {
        self.lexeme().into()
    }

    pub fn lexeme(&self) -> &str {
        match self.kind {
            TokenKind::String => &self.lexeme[1..self.lexeme.len() - 1],
            _ => self.lexeme,
        }
    }
}

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

    pub fn at_top_level(&self) -> bool {
        self.paren_level == 0
    }

    pub fn peek_token(&mut self) -> Result<Token<'a>, FroggiError> {
        if self.token.is_none() {
            self.next()?;
        }

        Ok(self.token.unwrap())
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, FroggiError> {
        if self.token.is_none() {
            self.next()?;
        }

        Ok(self.token.take().unwrap())
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
                            Err(FroggiError::parse(
                                crate::ParseError::UnbalancedParentheses,
                                self.line,
                            ))
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

    fn identifier(&mut self) -> Result<TokenKind, FroggiError> {
        while !is_control_character(self.peek()) {
            self.advance();
        }

        match self.lexeme()? {
            "box" => Ok(TokenKind::Box),
            "vbox" => Ok(TokenKind::VBox),
            "text" => Ok(TokenKind::Text),
            "inline" => Ok(TokenKind::Inline),
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

    fn text(&mut self) -> Result<TokenKind, FroggiError> {
        let start_line = self.line;

        while !self.at_end() && (self.peek() != b'\'' && self.peek() != b'"') {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            if self.peek() == b'\\' {
                self.advance();
                if self.peek() != b'\'' || self.peek() != b'"' {
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
