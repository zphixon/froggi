use crate::{FroggiError, ParseError};

use super::scan::{Scanner, Token, TokenKind};
use super::AstNode;

pub fn parse(data: &str) -> Result<Vec<AstNode<'_>>, Vec<FroggiError>> {
    let mut errors = Vec::new();
    let mut items = Vec::new();

    let mut scanner = Scanner::new(data);
    while scanner.peek_token(0)?.kind() != TokenKind::End {
        print!("parse {:?} - ", scanner.peek_token(0)?);
        match s_expr(&mut scanner) {
            Ok(item) => {
                println!("{:?}", item);
                items.push(item);
            }
            Err(error) => {
                println!("{:?}", error);
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        Ok(items)
    } else {
        Err(errors)
    }
}

fn s_expr<'a>(scanner: &mut Scanner<'a>) -> Result<AstNode<'a>, FroggiError> {
    consume(scanner, TokenKind::LeftParen)?;
    let token = scanner.next_token()?;
    let item = match token.kind() {
        TokenKind::Builtin => {
            match token.lexeme() {
                "txt" => parse_text_item(scanner, AstNode::text)?,
                "box" => parse_layout_box_item(scanner, AstNode::box_)?,
                "hbox" => parse_layout_box_item(scanner, AstNode::hbox)?,
                "img" => parse_text_item(scanner, AstNode::image)?,
                "page" => AstNode::empty(), // skip for now
                _ => {
                    return Err(FroggiError::parse(
                        ParseError::UnknownBuiltin {
                            builtin: String::from(token.lexeme()),
                        },
                        token.line(),
                    ))
                }
            }
        }
        _ => {
            //balance_parens(scanner)?;
            return Err(FroggiError::parse(
                ParseError::ExpectedBuiltin { got: token.kind() },
                token.line(),
            ));
        }
    };
    consume(scanner, TokenKind::RightParen)?;
    Ok(item)
}

fn parse_text_item<'a, F>(scanner: &mut Scanner<'a>, ctor: F) -> Result<AstNode<'a>, FroggiError>
where
    F: Fn(Token<'a>, Vec<Token<'a>>) -> AstNode<'a>
{
    let styles = parse_styles(scanner)?;
    let text = consume(scanner, TokenKind::Text)?;
    Ok(ctor(text, styles))
}

fn parse_layout_box_item<'a, F>(scanner: &mut Scanner<'a>, ctor: F) -> Result<AstNode<'a>, FroggiError>
where
    F: Fn(Vec<AstNode<'a>>, Vec<Token<'a>>) -> AstNode<'a>
{
    let styles = parse_styles(scanner)?;
    let mut children = vec![s_expr(scanner)?];
    if scanner.peek_token(0)?.kind() == TokenKind::LeftParen {
        children.push(s_expr(scanner)?);
    }
    Ok(ctor(children, styles))
}

fn parse_styles<'a>(scanner: &mut Scanner<'a>) -> Result<Vec<Token<'a>>, FroggiError> {
    let mut styles = vec![];
    loop {
        let token = scanner.peek_token(0)?;
        match token.kind() {
            // negation?
            TokenKind::User
            | TokenKind::ForegroundColor
            | TokenKind::FontChoice
            | TokenKind::Fill
            | TokenKind::BackgroundColor => styles.push(token),
            _ => break
        }
        scanner.next_token()?;
    }

    Ok(styles)
}

fn consume<'a>(scanner: &mut Scanner<'a>, kind: TokenKind) -> Result<Token<'a>, FroggiError> {
    let token = scanner.next_token()?;

    if token.kind() == kind {
        Ok(token)
    } else {
        Err(FroggiError::parse(
            ParseError::UnexpectedToken {
                expected: kind,
                got: token.kind(),
            },
            token.line(),
        ))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn sample() {
        let sample = include_str!("../../../server/pages/index.fml");
        super::super::scan::lex(sample).unwrap();
    }
}
