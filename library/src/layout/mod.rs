pub mod color;
pub mod font;
pub mod style;

use crate::markup::{ItemPayload, PageItem};
use crate::{FroggiError, MarkupError};

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
    Blob { name: String, alt: String },
    Link { text: String, location: String },
    Anchor { location: String },
    Empty,
}

impl LayoutItemKind {
    fn from_page_item(_page_item: &PageItem) -> Result<Self, FroggiError> {
        Ok(LayoutItemKind::Empty)
    }
}

#[derive(Debug)]
pub struct Document(Vec<style::Style>, Vec<LayoutItemKind>); // TODO: temporary

impl Document {
    pub fn new(data: &str) -> Result<Document, Vec<FroggiError>> {
        let page = crate::markup::parse::parse(data)?;

        let mut styles = Vec::new();
        for page_style in &page.page_styles {
            styles.push(style::Style::from_page_style(page_style)?);
        }

        let mut items = Vec::new();
        for page_item in &page.items {
            items.push(LayoutItemKind::from_page_item(page_item)?);
        }

        Ok(Document(styles, items))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn long() {
        let page = include_str!("../../../server/pages/long.fml");
        Document::new(page).unwrap();
    }

    #[test]
    fn text_with_children() {
        let page = r#"(text ("haha") ("nerd"))"#;
        assert!(Document::new(page).is_err());
    }

    #[test]
    fn box_with_text() {
        let page = r#"(box "haha nerd")"#;
        assert!(Document::new(page).is_err());
    }

    #[test]
    fn dont_hit_unreachable() {
        let page = r#"(# ("children") ("bad"))"#;
        assert!(Document::new(page).is_err());

        let page = r#"(^ ("children") ("bad"))"#;
        assert!(Document::new(page).is_err());

        let page = r#"(& ("children") ("bad"))"#;
        assert!(Document::new(page).is_err());
    }

    #[test]
    fn some_styles() {
        let page = r#"{(something (fg "23320a")) (^ (bg "2aaaaa"))}"#;
        Document::new(page).unwrap();
    }
}
