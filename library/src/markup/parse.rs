use crate::{AddMsg, FroggiError, ParseError};

use super::scan::{Scanner, Token, TokenKind};
use super::{InlineStyle, ItemPayload, Page, PageItem, PageStyle, PageStyleSelector};

/// Parse some data into a Page.
pub fn parse(data: &str) -> Result<Page<'_>, Vec<FroggiError>> {
    let mut errors = Vec::new();
    let mut items = Vec::new();
    let mut page_styles = Vec::new();

    let mut scanner = Scanner::new(data);
    while scanner.peek_token(0)?.kind() != TokenKind::End {
        match scanner.peek_token(0)?.kind() {
            // there should only be a single page-level style element
            TokenKind::LeftBrace if page_styles.is_empty() => match parse_page_styles(&mut scanner)
            {
                Ok(styles) => page_styles = styles,
                Err(error) => errors.push(error),
            },

            TokenKind::LeftParen => match parse_item(&mut scanner) {
                Ok(item) => {
                    items.push(item);
                }
                Err(error) => {
                    errors.push(error);
                }
            },

            _ => errors.push(FroggiError::parse(
                ParseError::ExpectedItem {
                    got: scanner.peek_token(0)?.clone_lexeme(),
                },
                scanner.peek_token(0)?.line(),
            )),
        }
    }

    if errors.is_empty() {
        Ok(Page { page_styles, items })
    } else {
        Err(errors)
    }
}

// consume top-level page style
fn parse_page_styles<'a>(scanner: &mut Scanner<'a>) -> Result<Vec<PageStyle<'a>>, FroggiError> {
    // parse outer list of rules
    let left_brace = consume(scanner, TokenKind::LeftBrace)?;

    let mut page_styles = Vec::new();

    while scanner.peek_token(0)?.kind() != TokenKind::RightBrace {
        // parse one single rule
        consume(scanner, TokenKind::LeftParen)?;

        // name of the rule
        let token = scanner.next_token()?;
        let selector = match token.kind() {
            TokenKind::Colon => PageStyleSelector::Builtin {
                name: consume(scanner, TokenKind::Identifier)?,
            },

            TokenKind::Identifier => PageStyleSelector::UserDefined { name: token },

            _ => {
                return Err(FroggiError::parse(
                    ParseError::UnexpectedToken {
                        expected: TokenKind::Identifier,
                        got: token.clone_lexeme(),
                    },
                    token.line(),
                ))
            }
        };

        // styles that belong to the rule
        let mut styles = Vec::new();
        while scanner.peek_token(0)?.kind() != TokenKind::RightParen {
            let token = scanner.next_token()?;
            match token.kind() {
                TokenKind::Identifier => {
                    styles.push(InlineStyle::NoArgs { name: token });
                }

                TokenKind::LeftParen => {
                    styles.push(InlineStyle::Arg {
                        name: consume(scanner, TokenKind::Identifier)?,
                        arg: consume(scanner, TokenKind::Text)?,
                    });
                    consume(scanner, TokenKind::RightParen)?;
                }

                _ => {
                    return Err(FroggiError::parse(
                        ParseError::ExpectedStyle {
                            got: token.clone_lexeme(),
                        },
                        token.line(),
                    ))
                }
            }
        }

        page_styles.push(PageStyle { selector, styles });
        consume(scanner, TokenKind::RightParen)?;
    }

    consume(scanner, TokenKind::RightBrace).msg(format!(
        "unbalanced braces starting on line {}",
        left_brace.line()
    ))?;

    Ok(page_styles)
}

// parse some normal page item
fn parse_item<'a>(scanner: &mut Scanner<'a>) -> Result<PageItem<'a>, FroggiError> {
    let left_paren = consume(scanner, TokenKind::LeftParen)?;

    let builtin = parse_builtin(scanner)?;
    let defined_styles = parse_defined_styles(scanner)?;
    let inline_styles = parse_inline_styles(scanner)?;
    let payload = parse_payload(scanner)?;

    consume(scanner, TokenKind::RightParen).msg(format!(
        "unbalanced parens starting on line {}",
        left_paren.line()
    ))?;

    Ok(PageItem {
        builtin,
        defined_styles,
        inline_styles,
        payload,
    })
}

fn parse_builtin<'a>(scanner: &mut Scanner<'a>) -> Result<Option<Token<'a>>, FroggiError> {
    Ok(if scanner.peek_token(0)?.kind() == TokenKind::Colon {
        consume(scanner, TokenKind::Colon)?;
        Some(consume(scanner, TokenKind::Identifier)?)
    } else {
        None
    })
}

fn parse_defined_styles<'a>(scanner: &mut Scanner<'a>) -> Result<Vec<Token<'a>>, FroggiError> {
    let mut defined_styles = Vec::new();
    while scanner.peek_token(0)?.kind() == TokenKind::Identifier {
        defined_styles.push(consume(scanner, TokenKind::Identifier)?);
    }
    Ok(defined_styles)
}

fn parse_inline_styles<'a>(scanner: &mut Scanner<'a>) -> Result<Vec<InlineStyle<'a>>, FroggiError> {
    let mut inline_styles = Vec::new();
    if scanner.peek_token(0)?.kind() == TokenKind::LeftBrace {
        consume(scanner, TokenKind::LeftBrace)?;

        while scanner.peek_token(0)?.kind() != TokenKind::RightBrace {
            let token = scanner.next_token()?;
            match token.kind() {
                TokenKind::Identifier => {
                    inline_styles.push(InlineStyle::NoArgs { name: token });
                }

                TokenKind::LeftParen => {
                    inline_styles.push(InlineStyle::Arg {
                        name: consume(scanner, TokenKind::Identifier)?,
                        arg: consume(scanner, TokenKind::Text)?,
                    });
                    consume(scanner, TokenKind::RightParen)?;
                }

                _ => {
                    return Err(FroggiError::parse(
                        ParseError::ExpectedStyle {
                            got: token.clone_lexeme(),
                        },
                        token.line(),
                    ))
                }
            }
        }

        consume(scanner, TokenKind::RightBrace)?;
    }

    Ok(inline_styles)
}

fn parse_payload<'a>(scanner: &mut Scanner<'a>) -> Result<ItemPayload<'a>, FroggiError> {
    if scanner.peek_token(0)?.kind() == TokenKind::Text {
        let mut text = Vec::new();
        while scanner.peek_token(0)?.kind() != TokenKind::RightParen {
            text.push(consume(scanner, TokenKind::Text)?);
        }

        Ok(ItemPayload::Text { text })
    } else if scanner.peek_token(0)?.kind() == TokenKind::LeftParen {
        // parse_item takes care of the left and right parens
        let mut children = Vec::new();
        while scanner.peek_token(0)?.kind() != TokenKind::RightParen {
            children.push(parse_item(scanner)?);
        }

        Ok(ItemPayload::Children { children })
    } else {
        Err(FroggiError::parse(
            ParseError::UnexpectedToken {
                expected: TokenKind::Text,
                got: scanner.peek_token(0)?.clone_lexeme(),
            },
            scanner.peek_token(0)?.line(),
        ))
    }
}

fn consume<'a>(scanner: &mut Scanner<'a>, kind: TokenKind) -> Result<Token<'a>, FroggiError> {
    let token = scanner.next_token()?;

    if token.kind() == kind {
        Ok(token)
    } else {
        Err(FroggiError::parse(
            ParseError::UnexpectedToken {
                expected: kind,
                got: token.clone_lexeme(),
            },
            token.line(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample() {
        let sample = include_str!("../../../server/pages/index.fml");
        crate::markup::scan::lex(sample).unwrap();
    }

    #[test]
    fn random_markup() {
        // currently invalid, but it will parse correctly
        let markup = r#"(hbox %328 (hbox $serif (hbox (box *ab0cad (hbox (box (hbox
            (hbox (box (hbox SomeThing (txt "Some text here, dope")))))
            ))))))"#;

        parse(markup).unwrap();
    }

    #[test]
    fn test_markup() {
        let sample = include_str!("../../../server/pages/test_markup.fml");
        parse(sample).unwrap();
    }
}
