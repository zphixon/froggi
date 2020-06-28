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
    fn from_page_item(page_item: &PageItem) -> Result<Self, FroggiError> {
        if let Some(builtin) = page_item.builtin {
            match builtin.lexeme() {
                // link items
                "^" => match &page_item.payload {
                    ItemPayload::Link { link, text } => {
                        // add the link to the document items
                        Ok(LayoutItemKind::Link {
                            // if the link doesn't have display text, copy the link location
                            text: if text.is_empty() {
                                link.clone_lexeme()
                            } else {
                                // otherwise collect the text
                                text.iter().fold(String::new(), |acc, next| {
                                    format!("{}{}", acc, next.lexeme())
                                })
                            },
                            location: link.clone_lexeme(),
                        })
                    }
                    // unreachable because the parser will never produce a link
                    // element without the correct payload variant
                    _ => unreachable!("link item isn't an ItemPayload::Link"),
                },

                // blob items
                "&" => match &page_item.payload {
                    ItemPayload::Blob { name, alt } => {
                        // add the blob to the document items
                        Ok(LayoutItemKind::Blob {
                            name: name.clone_lexeme(),
                            // collect the alt text - will result in an empty string if
                            // there isn't any alt text
                            alt: alt.iter().fold(String::new(), |acc, next| {
                                format!("{}{}", acc, next.lexeme())
                            }),
                        })
                    }
                    // unreachable because the parser will never produce a blob
                    // element without the correct payload variant
                    _ => unreachable!("blob item isn't an ItemPayload::Blob"),
                },

                "#" => match &page_item.payload {
                    ItemPayload::Anchor { anchor } => Ok(LayoutItemKind::Anchor {
                        location: anchor.clone_lexeme(),
                    }),
                    // unreachable because the parser will never produce an anchor
                    // element without the correct payload variant
                    _ => unreachable!("anchor item isn't an ItemPayload::Anchor"),
                },

                "box" => match &page_item.payload {
                    ItemPayload::Children { children, .. } => {
                        let mut childrens = Vec::new();
                        for child in children {
                            childrens.push(LayoutItem {
                                // TODO: get and propagate style
                                style: style::Style::default(),
                                kind: LayoutItemKind::from_page_item(child)?,
                            });
                        }

                        Ok(LayoutItemKind::Box {
                            children: childrens,
                        })
                    }

                    _ => Err(FroggiError::markup(
                        MarkupError::ExpectedChildren {
                            item: builtin.clone_lexeme(),
                        },
                        builtin.line(),
                    )),
                },

                "vbox" => match &page_item.payload {
                    ItemPayload::Children { children, .. } => {
                        let mut childrens = Vec::new();
                        for child in children {
                            childrens.push(LayoutItem {
                                // TODO: get and propagate style
                                style: style::Style::default(),
                                kind: LayoutItemKind::from_page_item(child)?,
                            });
                        }

                        Ok(LayoutItemKind::VBox {
                            children: childrens,
                        })
                    }

                    _ => Err(FroggiError::markup(
                        MarkupError::ExpectedChildren {
                            item: builtin.clone_lexeme(),
                        },
                        builtin.line(),
                    )),
                },

                "text" => match &page_item.payload {
                    ItemPayload::Text { text } => Ok(LayoutItemKind::Text {
                        text: text.iter().fold(String::new(), |acc, next| {
                            format!("{}{}", acc, next.lexeme())
                        }),
                    }),

                    _ => Err(FroggiError::markup(
                        MarkupError::ExpectedText {
                            item: builtin.clone_lexeme(),
                        },
                        builtin.line(),
                    )),
                },

                _ => Err(FroggiError::markup(
                    MarkupError::UnknownBuiltin {
                        builtin: builtin.clone_lexeme(),
                    },
                    builtin.line(),
                )),
            }
        } else {
            match &page_item.payload {
                // collect the text of a text item
                ItemPayload::Text { text } => Ok(LayoutItemKind::Text {
                    text: text.iter().fold(String::new(), |acc, next| {
                        format!("{}{}", acc, next.lexeme())
                    }),
                }),

                // error for everything else
                ItemPayload::Children { line, .. } => Err(FroggiError::markup(
                    MarkupError::ExpectedText {
                        item: String::from("implicit text item"),
                    },
                    *line,
                )),
                ItemPayload::Link { link, .. } => Err(FroggiError::markup(
                    MarkupError::ExpectedText {
                        item: String::from("implicit text item"),
                    },
                    link.line(),
                )),
                ItemPayload::Blob { name, .. } => Err(FroggiError::markup(
                    MarkupError::ExpectedText {
                        item: String::from("implicit text item"),
                    },
                    name.line(),
                )),
                ItemPayload::Anchor { anchor } => Err(FroggiError::markup(
                    MarkupError::ExpectedText {
                        item: String::from("implicit text item"),
                    },
                    anchor.line(),
                )),
            }
        }
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
}
