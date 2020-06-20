pub mod ast;
pub mod parse;
pub mod scan;
use scan::Token;

#[derive(Debug)]
pub struct PageItem<'a> {
    // a None value implies a :text item
    pub builtin: Option<Token<'a>>,
    pub defined_styles: Vec<Token<'a>>,
    pub inline_styles: Vec<InlineStyle<'a>>,
    pub payload: ItemPayload<'a>,
}

#[derive(Debug)]
pub enum ItemPayload<'a> {
    Text { text: Vec<Token<'a>> },
    Children { children: Vec<PageItem<'a>> },
}

#[derive(Debug)]
pub enum InlineStyle<'a> {
    NoArgs { name: Token<'a> },
    Arg { name: Token<'a>, arg: Token<'a> },
}
