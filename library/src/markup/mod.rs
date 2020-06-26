pub mod ast;
pub mod parse;
pub mod scan;

use scan::Token;

#[derive(Debug, PartialEq)]
pub struct Page<'a> {
    pub page_styles: Vec<PageStyle<'a>>,
    pub items: Vec<PageItem<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PageStyle<'a> {
    pub selector: Token<'a>,
    pub styles: Vec<InlineStyle<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PageItem<'a> {
    // a None value implies a :text item
    pub builtin: Option<Token<'a>>,
    pub inline_styles: Vec<InlineStyle<'a>>,
    pub payload: ItemPayload<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ItemPayload<'a> {
    Text { text: Vec<Token<'a>> },
    Children { children: Vec<PageItem<'a>> },
    Reference { reference: ReferenceKind<'a> },
}

#[derive(Debug, PartialEq)]
pub enum ReferenceKind<'a> {
    Link {
        link: Token<'a>,
        text: Vec<Token<'a>>,
    },
    Blob {
        name: Token<'a>,
        alt: Vec<Token<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct WithArg<'a> {
    pub name: Token<'a>,
    pub arg: Token<'a>,
}

#[derive(Debug, PartialEq)]
pub struct WithoutArg<'a> {
    pub name: Token<'a>,
}

#[derive(Debug, PartialEq)]
pub enum InlineStyle<'a> {
    WithoutArg(WithoutArg<'a>),
    WithArg(WithArg<'a>),
}

impl InlineStyle<'_> {
    pub fn name(&self) -> &str {
        match self {
            InlineStyle::WithoutArg(without) => without.name.lexeme(),
            InlineStyle::WithArg(with) => with.name.lexeme(),
        }
    }

    pub fn has_arg(&self) -> bool {
        match self {
            InlineStyle::WithoutArg(_) => false,
            InlineStyle::WithArg(_) => true,
        }
    }

    pub fn token(&self) -> &Token<'_> {
        match self {
            InlineStyle::WithoutArg(without) => &without.name,
            InlineStyle::WithArg(with) => &with.name,
        }
    }

    pub fn as_with_arg(&self) -> &WithArg {
        match self {
            InlineStyle::WithArg(with) => &with,
            _ => panic!("as_with_arg on InlineStyle::WithoutArg"),
        }
    }

    pub fn as_without_arg(&self) -> &WithoutArg {
        match self {
            InlineStyle::WithoutArg(without) => &without,
            _ => panic!("as_without_arg on InlineStyle::WithArg"),
        }
    }
}
