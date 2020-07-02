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
    pub builtin: Token<'a>,
    pub inline_styles: Vec<InlineStyle<'a>>,
    pub payload: ItemPayload<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ItemPayload<'a> {
    Text {
        text: Vec<Token<'a>>,
    },
    Children {
        children: Vec<PageItem<'a>>,
        line: usize,
    },
    Link {
        link: Token<'a>,
        text: Vec<Token<'a>>,
    },
    Blob {
        name: Token<'a>,
        alt: Vec<Token<'a>>,
    },
    Anchor {
        anchor: Token<'a>,
    },
}

#[derive(Debug, PartialEq)]
pub enum InlineStyle<'a> {
    Mono { token: Token<'a> },
    Serif { token: Token<'a> },
    Sans { token: Token<'a> },
    Bold { token: Token<'a> },
    Italic { token: Token<'a> },
    Underline { token: Token<'a> },
    Strike { token: Token<'a> },
    Fg { token: Token<'a>, arg: Token<'a> },
    Bg { token: Token<'a>, arg: Token<'a> },
    Fill { token: Token<'a>, arg: Token<'a> },
    Size { token: Token<'a>, arg: Token<'a> },
    UserDefined { token: Token<'a> },
}
