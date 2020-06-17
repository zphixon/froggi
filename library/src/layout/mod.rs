pub mod color;
pub mod font;
pub mod style;

use crate::FroggiError;

use style::Style;

/// AST node. Used by a client to decide the layout of the page.
pub enum LayoutItem<'a> {
    Box {
        style: Option<Style>,
        children: Vec<LayoutItem<'a>>,
    },
    HBox {
        style: Option<Style>,
        children: Vec<LayoutItem<'a>>,
    },
    Text {
        style: Option<Style>,
        text: &'a str,
    },
    Image(&'a str),
    Empty,
}

/// A FML document.
pub struct Document<'a> {
    pub title: &'a str,
    pub base_style: Style,
    pub tree: Vec<LayoutItem<'a>>,
}

impl Document<'_> {
    pub fn new(data: &str) -> Result<Document<'_>, Vec<FroggiError>> {
        Ok(Document {
            title: data,
            base_style: Style::default(),
            tree: make_layout(data)?,
        })
    }
}

fn make_layout(data: &str) -> Result<Vec<LayoutItem<'_>>, Vec<FroggiError>> {
    let r = crate::markup::parse::parse(data)?;
    panic!("wow it worked {:#?}", r);
}
