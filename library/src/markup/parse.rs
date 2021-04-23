use crate::{AddMsg, FroggiError, ParseError};

use super::scan::{Scanner, Token, TokenKind};
use super::{ExpressionPayload, InlineStyle, PageAst, PageExpressionAst, PageStyles};

use std::collections::HashMap;

/// Parse some data into a Page.
pub fn parse(data: &str) -> Result<PageAst<'_>, Vec<FroggiError>> {
    let mut errors = Vec::new();
    let mut expressions = Vec::new();
    let mut page_styles = HashMap::new();

    let mut first_expression = true;
    let mut scanner = Scanner::new(data);
    while scanner.peek_token()?.kind() != TokenKind::End {
        match scanner.peek_token()?.kind() {
            // there should only be a single page-level style element,
            // and it should be the first expression
            TokenKind::LeftBrace if first_expression => {
                first_expression = false;
                match parse_page_styles(&mut scanner) {
                    Ok(styles) => page_styles = styles,
                    Err(error) => errors.push(error),
                }
            }

            TokenKind::LeftBrace if !first_expression => {
                errors.push(
                    FroggiError::parse(
                        ParseError::ExpectedExpression { got: "{".into() },
                        scanner.peek_token()?.line(),
                    )
                    .msg_str("page style expression must be the first expression in the page"),
                );
                while scanner.peek_token()?.kind() != TokenKind::RightBrace {
                    scanner.next_token()?;
                }
                scanner.next_token()?;
            }

            TokenKind::LeftParen => {
                first_expression = false;
                match parse_expression(&mut scanner, &page_styles) {
                    Ok(expression) => {
                        expressions.push(expression);
                    }
                    Err(error) => {
                        errors.push(error);
                    }
                }
            }

            _ => {
                errors.push(FroggiError::parse(
                    ParseError::ExpectedExpression {
                        got: scanner.peek_token()?.clone_lexeme(),
                    },
                    scanner.peek_token()?.line(),
                ));
                scanner.next_token()?;
                while !scanner.at_top_level() {
                    scanner.next_token()?;
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(PageAst {
            styles: page_styles,
            expressions,
        })
    } else {
        Err(errors)
    }
}

// consume top-level page style
fn parse_page_styles<'a>(scanner: &mut Scanner<'a>) -> Result<PageStyles<'a>, FroggiError> {
    // parse outer list of rules
    let left_brace = consume(scanner, TokenKind::LeftBrace)?;

    let in_page_style_expression = true;
    let mut page_styles = HashMap::new();

    while scanner.peek_token()?.kind() != TokenKind::RightBrace {
        // parse one single rule
        consume(scanner, TokenKind::LeftParen)
            .msg_str("expected style rules inside page style expression")?;

        // name of the rule
        let selector = consume_selector(scanner)?;

        // styles that belong to the rule
        let styles = parse_style_list(
            scanner,
            &HashMap::with_capacity(0),
            in_page_style_expression,
        )?;

        page_styles.insert(selector, styles);
        consume(scanner, TokenKind::RightParen).msg_str("end of the style rule")?;
    }

    consume(scanner, TokenKind::RightBrace).msg(format!(
        "unbalanced braces starting on line {}",
        left_brace.line()
    ))?;

    Ok(page_styles)
}

// parse some normal page expression
fn parse_expression<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
) -> Result<PageExpressionAst<'a>, FroggiError> {
    let left_paren = consume(scanner, TokenKind::LeftParen)?;

    let result = match scanner.peek_token()?.kind() {
        TokenKind::Blob => parse_blob(scanner, page_styles)?,
        TokenKind::Link => parse_link(scanner, page_styles)?,
        TokenKind::Anchor => parse_anchor(scanner)?,
        TokenKind::Tall => parse_child(scanner, page_styles, TokenKind::Tall)?,
        TokenKind::Wide => parse_child(scanner, page_styles, TokenKind::Wide)?,
        TokenKind::Inline => parse_child(scanner, page_styles, TokenKind::Inline)?,
        _ => parse_implicit_text(scanner, page_styles)?,
    };

    consume(scanner, TokenKind::RightParen).msg(format!(
        "unbalanced parens starting on line {}",
        left_paren.line()
    ))?;

    Ok(result)
}

fn parse_blob<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
) -> Result<PageExpressionAst<'a>, FroggiError> {
    let builtin = consume(scanner, TokenKind::Blob)?;
    let name = consume(scanner, TokenKind::String)?;

    let styles = parse_inline_styles(scanner, page_styles)?;
    let payload = ExpressionPayload::Blob {
        name,
        alt: collect_text(scanner)?,
    };

    Ok(PageExpressionAst {
        builtin,
        styles,
        payload,
    })
}

fn parse_link<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
) -> Result<PageExpressionAst<'a>, FroggiError> {
    let builtin = consume(scanner, TokenKind::Link)?;
    let link = consume(scanner, TokenKind::String)?;

    let styles = parse_inline_styles(scanner, page_styles)?;
    let payload = ExpressionPayload::Link {
        link,
        text: collect_text(scanner)?,
    };

    Ok(PageExpressionAst {
        builtin,
        styles,
        payload,
    })
}

fn parse_anchor<'a>(scanner: &mut Scanner<'a>) -> Result<PageExpressionAst<'a>, FroggiError> {
    let builtin = consume(scanner, TokenKind::Anchor)?;
    let anchor = consume(scanner, TokenKind::String)?;
    let payload = ExpressionPayload::Anchor { anchor };
    Ok(PageExpressionAst {
        builtin,
        styles: Vec::new(),
        payload,
    })
}

fn parse_child<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
    kind: TokenKind,
) -> Result<PageExpressionAst<'a>, FroggiError> {
    let builtin = consume(scanner, kind)?;
    let styles = parse_inline_styles(scanner, page_styles)?;
    let mut children = Vec::new();

    while scanner.peek_token()?.kind() != TokenKind::RightParen {
        children.push(parse_expression(scanner, page_styles)?);
    }

    Ok(PageExpressionAst {
        builtin,
        styles,
        payload: ExpressionPayload::Children {
            children,
            line: builtin.line(),
        },
    })
}

fn parse_implicit_text<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
) -> Result<PageExpressionAst<'a>, FroggiError> {
    let implicit = Token::new(TokenKind::Text, scanner.peek_token()?.line(), "");
    let styles = parse_inline_styles(scanner, page_styles)?;
    let text = collect_text(scanner)?;

    Ok(PageExpressionAst {
        builtin: implicit,
        styles,
        payload: ExpressionPayload::Text { text },
    })
}

fn parse_inline_styles<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
) -> Result<Vec<InlineStyle<'a>>, FroggiError> {
    let in_page_style_expression = false;
    if scanner.peek_token()?.kind() == TokenKind::LeftBrace {
        consume(scanner, TokenKind::LeftBrace)?;
        let inline_styles = parse_style_list(scanner, page_styles, in_page_style_expression)?;
        consume(scanner, TokenKind::RightBrace).msg_str("expected the end of the inline style")?;
        Ok(inline_styles)
    } else {
        Ok(Vec::new())
    }
}

fn parse_style_list<'a>(
    scanner: &mut Scanner<'a>,
    page_styles: &PageStyles<'a>,
    in_page_style_expression: bool,
) -> Result<Vec<InlineStyle<'a>>, FroggiError> {
    let mut styles = Vec::new();

    // this could be called from either inline style parsing or page style parsing
    while scanner.peek_token()?.kind() != TokenKind::RightBrace
        && scanner.peek_token()?.kind() != TokenKind::RightParen
    {
        let token = scanner.next_token()?;
        match token.kind() {
            TokenKind::Identifier => {
                if !in_page_style_expression {
                    if page_styles.contains_key(&token) {
                        styles.push(InlineStyle::UserDefined { token });
                    } else {
                        return Err(FroggiError::parse(
                            ParseError::UnknownStyle {
                                style: token.clone_lexeme(),
                            },
                            token.line(),
                        ));
                    }
                } else {
                    // if we're in the page style expression, we've already consumed the selector
                    return Err(FroggiError::parse(
                        ParseError::RecursiveStyle {
                            style: token.clone_lexeme(),
                        },
                        token.line(),
                    ))
                    .msg_str("styles may not reference user-defined styles.");
                }
            }

            TokenKind::Mono => {
                styles.push(InlineStyle::Mono { token });
            }

            TokenKind::Serif => {
                styles.push(InlineStyle::Serif { token });
            }

            TokenKind::Sans => {
                styles.push(InlineStyle::Sans { token });
            }

            TokenKind::Bold => {
                styles.push(InlineStyle::Bold { token });
            }

            TokenKind::Italic => {
                styles.push(InlineStyle::Italic { token });
            }

            TokenKind::Underline => {
                styles.push(InlineStyle::Underline { token });
            }

            TokenKind::Strike => {
                styles.push(InlineStyle::Strike { token });
            }

            TokenKind::LeftParen => {
                let token = scanner.next_token()?;
                let arg = consume(scanner, TokenKind::String).msg(format!(
                    "expected an argument for the style {}",
                    token.lexeme()
                ))?;

                match token.kind() {
                    TokenKind::Fg => {
                        let arg = parse_hex_color(arg)?;
                        styles.push(InlineStyle::Fg { token, arg });
                    }

                    TokenKind::Bg => {
                        let arg = parse_hex_color(arg)?;
                        styles.push(InlineStyle::Bg { token, arg });
                    }

                    TokenKind::Fill => {
                        let arg = arg.lexeme().parse::<f32>().map_err(|_| {
                            FroggiError::parse(
                                ParseError::IncorrectNumberFormat {
                                    num: arg.clone_lexeme(),
                                    wanted: String::from("1 or more"),
                                },
                                arg.line(),
                            )
                        })?;

                        styles.push(InlineStyle::Fill { token, arg });
                    }

                    TokenKind::Size => {
                        let arg = arg.lexeme().parse::<usize>().map_err(|_| {
                            FroggiError::parse(
                                ParseError::IncorrectNumberFormat {
                                    num: arg.clone_lexeme(),
                                    wanted: String::from("valid size"),
                                },
                                arg.line(),
                            )
                        })?;

                        styles.push(InlineStyle::Size { token, arg });
                    }

                    _ => {
                        return Err(FroggiError::parse(
                            ParseError::ExpectedStyle {
                                got: token.clone_lexeme(),
                            },
                            token.line(),
                        ))
                        .msg(format!("{} does not take an argument", token.lexeme()))
                    }
                }

                consume(scanner, TokenKind::RightParen)?;
            }

            _ => {
                return Err(FroggiError::parse(
                    ParseError::ExpectedStyle {
                        got: token.clone_lexeme(),
                    },
                    token.line(),
                ))
                .msg_str("expected a style rule in the list of inline style rules")
            }
        }
    }

    Ok(styles)
}

fn parse_hex_color(arg: Token) -> Result<(u8, u8, u8), FroggiError> {
    // TODO: support hsv/rgb/3-digit hex values?
    let bytes = hex::decode(arg.lexeme()).map_err(|_| {
        FroggiError::parse(
            ParseError::IncorrectNumberFormat {
                num: arg.clone_lexeme(),
                wanted: String::from("valid hex number"),
            },
            arg.line(),
        )
    })?;

    if bytes.len() != 3 {
        return Err(FroggiError::parse(
            ParseError::IncorrectNumberFormat {
                num: arg.clone_lexeme(),
                wanted: String::from("6-digit hex number"),
            },
            arg.line(),
        ));
    }

    Ok((bytes[0], bytes[1], bytes[2]))
}

fn collect_text<'a>(scanner: &mut Scanner<'a>) -> Result<Vec<Token<'a>>, FroggiError> {
    let mut text = Vec::new();

    while scanner.peek_token()?.kind() != TokenKind::RightParen {
        text.push(consume(scanner, TokenKind::String)?);
    }

    Ok(text)
}

fn consume_selector<'a>(scanner: &mut Scanner<'a>) -> Result<Token<'a>, FroggiError> {
    let token = scanner.next_token()?;
    match token.kind() {
        TokenKind::Identifier | TokenKind::Link | TokenKind::Wide | TokenKind::Tall => Ok(token),
        _ => Err(FroggiError::parse(
            ParseError::UnexpectedToken {
                expected: TokenKind::Identifier,
                got: token.clone_lexeme(),
            },
            token.line(),
        ))
        .msg_str("selectors must be either built-in expression types or links, or user-defined selectors"),
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
    fn recursive_user_style() {
        let page = "{(a b) (b a)}";

        match parse(page).unwrap_err()[0].kind() {
            crate::ErrorKind::ParseError { error, .. } => match error {
                ParseError::RecursiveStyle { .. } => {}
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn anchor() {
        let sample = r#"(# "something")"#;
        let page = parse(sample).unwrap();
        match page.expressions[0].payload {
            ExpressionPayload::Anchor { anchor } => assert_eq!(anchor.lexeme(), "something"),
            _ => panic!(),
        }
    }

    #[test]
    fn references() {
        let sample =
            r#"{(user-style)}(& "image.jpg" {user-style (fg "300300") serif} "with alt" " text")"#;
        parse(sample).unwrap();

        let sample = r#"(& "somewhere")"#;
        parse(sample).unwrap();
    }

    #[test]
    fn links() {
        let sample = r#"{(footnote)}(^ "frgi://www.lipsum.com/" {footnote (fill "20")} "from frgi://www.lipsum.com/")"#;
        parse(sample).unwrap();

        let sample = r#"(^ "frgi://www.lipsum.com/" {serif })"#;
        parse(sample).unwrap();

        let sample = r#"(^ "frgi://www.lipsum.com/")"#;
        parse(sample).unwrap();
    }

    #[test]
    fn parent_style_missing_arg() {
        let expression = r#"(box {(fg)} ("multiple children?") (style "why"))"#;

        match parse(expression).unwrap_err()[0].kind() {
            crate::ErrorKind::ParseError { error, .. } => match error {
                ParseError::UnexpectedToken { expected, .. } => match expected {
                    TokenKind::String => {}
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn child_style_missing_arg() {
        let expression = r#"{(style)} (box ({style (fg)} "why"))"#;

        match parse(expression).unwrap_err()[0].kind() {
            crate::ErrorKind::ParseError { error, .. } => match error {
                ParseError::UnexpectedToken { expected, .. } => match expected {
                    TokenKind::String => {}
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn page_style_missing_arg() {
        let style = "{(style (fg))}";

        match parse(style).unwrap_err()[0].kind() {
            crate::ErrorKind::ParseError { error, .. } => match error {
                ParseError::UnexpectedToken { expected, .. } => match expected {
                    TokenKind::String => {}
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn well_formed_page_expression() {
        let expression = r#"{(user-style (fg "333333")) (inline-style (bg "222222"))}(wide {user-style inline-style (fg "111111")} ("children") ({mono} "style"))"#;
        parse(expression).unwrap();
    }

    #[test]
    fn ill_formed_page_styles() {
        use crate::markup::scan::Scanner;

        // these .is_err()'s are OK since we know that they
        // can never fail for a reason that we don't expect
        let style = "{";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "{text) serif}";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "{(text) serif}";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "{() (style)}";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "{( (style)}";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "{ (style))}";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());

        let style = "";
        let mut scanner = Scanner::new(style);
        assert!(parse_page_styles(&mut scanner).is_err());
    }

    #[test]
    fn well_formed_page_style() {
        let style = r#"{(text serif)(footnote underline (fg "902100"))}"#;
        let mut scanner = crate::markup::scan::Scanner::new(style);
        let style = parse_page_styles(&mut scanner).unwrap();
        let mut styles = HashMap::new();
        styles.insert(
            Token::new(TokenKind::Identifier, 1, "text"),
            vec![InlineStyle::Serif {
                token: Token::new(TokenKind::Serif, 1, "serif"),
            }],
        );
        styles.insert(
            Token::new(TokenKind::Identifier, 1, "footnote"),
            vec![
                InlineStyle::Underline {
                    token: Token::new(TokenKind::Underline, 1, "underline"),
                },
                InlineStyle::Fg {
                    token: Token::new(TokenKind::Fg, 1, "fg"),
                    arg: (0x90, 0x21, 0x00),
                },
            ],
        );
        assert_eq!(style, styles);
    }

    #[test]
    fn test_markup() {
        let sample = include_str!("../../../server/pages/test_markup.fml");
        parse(sample).unwrap();
    }
}
