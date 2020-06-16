use crate::{FroggiError, ParseError};

use super::scan::{Scanner, Token, TokenKind};
use super::Ast;

pub fn parse(data: &str) -> Result<Vec<Ast>, Vec<FroggiError>> {
    let mut errors = Vec::new();
    let mut items = Vec::new();

    let mut scanner = Scanner::new(data);
    while scanner
        .current_token()
        .map(|token| token.kind() != TokenKind::End)
        .unwrap_or(true)
    {
        scanner.next_token()?;
        //match s_expr(&mut scanner) {
        //    Ok(item) => items.push(item),
        //    Err(error) => errors.push(error),
        //}
    }

    if errors.is_empty() {
        Ok(items)
    } else {
        Err(errors)
    }
}

//fn s_expr<'a>(scanner: &mut Scanner<'a>) -> Result<Item<'a>, FroggiError> {
//    consume(scanner, TokenKind::LeftParen)?;
//    let token = scanner.next_token()?;
//    let item = match token.kind() {
//        TokenKind::Builtin => {
//            match token.lexeme() {
//                "txt" => Item::Txt {
//                },
//                "box" => Item::Box,
//                "hbox" => Item::HBox,
//                "page" => Item::Empty, // skip for now
//                _ => return Err(FroggiError::parse(ParseError::UnknownBuiltin {
//                    builtin: String::from(token.lexeme())
//                })),
//            }
//        },
//        _ => {
//            balance_parens(scanner)?;
//            return Err(FroggiError::parse(ParseError::UnexpectedToken {
//                expected: TokenKind::Builtin,
//                got: token.kind()
//            }, token.line()))
//        },
//    }
//    consume(scanner, TokenKind::RightParen)?;
//    Ok(item)
//}

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

fn balance_parens<'a>(scanner: &mut Scanner<'a>) -> Result<(), FroggiError> {
    let start = scanner.current_token().unwrap().line();
    let mut current_level = 0;
    while scanner.current_token().unwrap().kind() != TokenKind::End {
        let token = scanner.next_token()?;
        match token.kind() {
            TokenKind::LeftParen => current_level += 1,
            TokenKind::RightParen => {
                if current_level == 0 {
                    break;
                }
                current_level -= 1;
            }
            _ => {}
        }
    }

    if scanner.current_token().unwrap().kind() == TokenKind::End {
        Err(FroggiError::parse(ParseError::UnbalancedParentheses, start))
    } else {
        Ok(())
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
