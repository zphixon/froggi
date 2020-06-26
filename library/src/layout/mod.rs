pub mod color;
pub mod font;
pub mod style;

use crate::FroggiError;

#[derive(Debug)]
pub struct LayoutItem {
    kind: LayoutItemKind,
    style: style::Style,
}

#[derive(Debug)]
pub enum LayoutItemKind {
    Box { children: Vec<LayoutItem> },
    VBox { children: Vec<LayoutItem> },
    Text { text: String },
    Image { name: String },
    Link { text: String, location: String },
    Empty,
}

#[derive(Debug)]
pub struct Document(Vec<LayoutItem>);

impl Document {
    pub fn new(data: &str) -> Result<Document, Vec<FroggiError>> {
        let page = crate::markup::parse::parse(data)?;

        let mut styles = Vec::new();
        for page_style in page.page_styles {
            styles.push(style::Style::from_page_style(page_style)?);
        }

        panic!("{:#?}", styles);
        // TODO: apply styles to page items

        Ok(Document(Vec::new()))
    }
}
