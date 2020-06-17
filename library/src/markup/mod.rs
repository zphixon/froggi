pub mod ast;
pub mod parse;
pub mod scan;

use scan::Token;

#[derive(Debug)]
pub struct AstNode<'a> {
    kind: AstKind<'a>,
    styles: Vec<Token<'a>>,
}

impl AstNode<'_> {
    fn text<'a>(text: Token<'a>, styles: Vec<Token<'a>>) -> AstNode<'a> {
        AstNode {
            kind: AstKind::Text { text },
            styles,
        }
    }

    fn box_<'a>(children: Vec<AstNode<'a>>, styles: Vec<Token<'a>>) -> AstNode<'a> {
        AstNode {
            kind: AstKind::Box { children },
            styles,
        }
    }

    fn hbox<'a>(children: Vec<AstNode<'a>>, styles: Vec<Token<'a>>) -> AstNode<'a> {
        AstNode {
            kind: AstKind::HBox { children },
            styles,
        }
    }

    fn image<'a>(filename: Token<'a>, styles: Vec<Token<'a>>) -> AstNode<'a> {
        AstNode {
            kind: AstKind::Image { filename },
            styles,
        }
    }

    fn empty<'a>() -> AstNode<'a> {
        AstNode {
            kind: AstKind::Empty,
            styles: Vec::with_capacity(0),
        }
    }
}

#[derive(Debug)]
pub enum AstKind<'a> {
    Text { text: Token<'a> },
    Box { children: Vec<AstNode<'a>> },
    HBox { children: Vec<AstNode<'a>> },
    Image { filename: Token<'a> },
    Empty,
}
