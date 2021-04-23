//! Page types that are easier to deal with than the raw AST.

use crate::markup::{ExpressionPayload, InlineStyle, Page, PageExpression, PageStyles};

use std::collections::HashMap;

/// An owned document. More useful than Page for drawing to the screen.
#[derive(Debug, PartialEq)]
pub struct Document {
    styles: HashMap<String, Style>,
    expressions: Vec<DocumentExpression>,
}

impl Document {
    pub fn from_page(page: &Page) -> Document {
        let mut document = Document {
            styles: HashMap::new(),
            expressions: Vec::new(),
        };

        for (name, styles) in page.styles.iter() {
            let mut style = Style::new();
            inline_styles_to_style(&styles, &HashMap::with_capacity(0), &mut style);
            document.styles.insert(name.clone_lexeme(), style);
        }

        for expression in page.expressions.iter() {
            document
                .expressions
                .push(DocumentExpression::from_page_expression(
                    expression,
                    &page.styles,
                ));
        }

        document
    }
}

/// An owned document expression.
#[derive(Debug, PartialEq)]
pub struct DocumentExpression {
    style: Style,
    direction: Direction,
    contents: DocumentExpressionContents,
}

impl DocumentExpression {
    fn from_page_expression(
        page_expression: &PageExpression,
        page_styles: &PageStyles,
    ) -> DocumentExpression {
        let mut style = Style::new();
        inline_styles_to_style(&page_expression.styles, page_styles, &mut style);
        let direction = page_expression.builtin.direction();
        let contents = DocumentExpressionContents::from_expression_payload(
            &page_expression.payload,
            page_styles,
        );

        DocumentExpression {
            style,
            direction,
            contents,
        }
    }
}

/// Contents of a document expression. Stored independently of its layout.
#[derive(Debug, PartialEq)]
pub enum DocumentExpressionContents {
    /// Plain text
    Text {
        /// The text of the expression
        text: String,
    },
    /// A link to another page or anchor on the same page
    Link {
        /// The display text of the link
        text: String,
        /// The URL target of the link
        url: String,
    },
    /// A reference to a page item by name
    Blob {
        /// Name of the item
        name: String,
        /// Alt text of the item, not optional
        alt: String,
    },
    /// An anchor point for expression insertion or link targeting
    Anchor {
        /// Name of the anchor
        name: String,
    },
    /// The expression has one or more sub-expressions
    Children {
        /// Children of the expression
        children: Vec<DocumentExpression>,
    },
}

impl DocumentExpressionContents {
    fn from_expression_payload(
        payload: &ExpressionPayload,
        page_styles: &PageStyles,
    ) -> DocumentExpressionContents {
        match payload {
            ExpressionPayload::Text { text } => DocumentExpressionContents::Text {
                text: text.iter().fold(String::new(), |mut string, token| {
                    string.push_str(token.lexeme());
                    string
                }),
            },
            ExpressionPayload::Children { children, .. } => DocumentExpressionContents::Children {
                children: children
                    .iter()
                    .map(|child| DocumentExpression::from_page_expression(child, page_styles))
                    .collect(),
            },
            ExpressionPayload::Link { link, text } => DocumentExpressionContents::Link {
                text: text.iter().fold(String::new(), |mut string, token| {
                    string.push_str(token.lexeme());
                    string
                }),
                url: link.clone_lexeme(),
            },
            ExpressionPayload::Blob { name, alt } => DocumentExpressionContents::Blob {
                name: name.clone_lexeme(),
                alt: alt.iter().fold(String::new(), |mut string, token| {
                    string.push_str(token.lexeme());
                    string
                }),
            },
            ExpressionPayload::Anchor { anchor } => DocumentExpressionContents::Anchor {
                name: anchor.clone_lexeme(),
            },
        }
    }
}

/// Direction for screen layout.
#[derive(Debug, PartialEq)]
pub enum Direction {
    /// Items are laid out horizontally
    Horizontal,
    /// Items are laid out vertically
    Vertical,
    /// Items are laid out inline
    Inline,
}

/// Type of font.
///
/// Created separately from page parsing.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontType {
    /// Monospace font
    Mono,
    /// Serif font
    Serif,
    /// Sans-serif font
    Sans,
}

/// Style of a font.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct FontStyle {
    /// Bold-face font
    pub bold: bool,
    /// Italic font
    pub italic: bool,
    /// Font is underlined
    pub underline: bool,
    /// Font is strikethrough
    pub strike: bool,
}

/// The style of an expression.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Style {
    /// The type of font
    pub font_type: FontType,
    /// The style of that font
    pub font_style: FontStyle,
    /// Background color
    pub background: (u8, u8, u8),
    /// Text color
    pub foreground: (u8, u8, u8),
    /// Horizontal or vertical width taken up
    pub fill: Option<f32>,
    /// Font size
    pub size: usize,
}

impl Style {
    fn new() -> Style {
        Style {
            font_type: FontType::Serif,
            font_style: FontStyle::default(),
            background: (0xff, 0xff, 0xff),
            foreground: (0x00, 0x00, 0x00),
            fill: None,
            size: 12,
        }
    }

    fn set_font_type(&mut self, font_type: FontType) {
        self.font_type = font_type;
    }

    fn set_bold(&mut self) {
        self.font_style.bold = true;
    }

    fn set_italic(&mut self) {
        self.font_style.italic = true;
    }

    fn set_underline(&mut self) {
        self.font_style.underline = true;
    }

    fn set_strike(&mut self) {
        self.font_style.strike = true;
    }

    fn set_background(&mut self, background: (u8, u8, u8)) {
        self.background = background;
    }

    fn set_foreground(&mut self, foreground: (u8, u8, u8)) {
        self.foreground = foreground;
    }

    fn set_fill(&mut self, fill: Option<f32>) {
        self.fill = fill;
    }

    fn set_size(&mut self, size: usize) {
        self.size = size;
    }
}

fn inline_styles_to_style(styles: &[InlineStyle], page_styles: &PageStyles, style: &mut Style) {
    for inline_style in styles {
        match inline_style {
            InlineStyle::Mono { .. } => style.set_font_type(FontType::Mono),
            InlineStyle::Serif { .. } => style.set_font_type(FontType::Serif),
            InlineStyle::Sans { .. } => style.set_font_type(FontType::Sans),
            InlineStyle::Bold { .. } => style.set_bold(),
            InlineStyle::Italic { .. } => style.set_italic(),
            InlineStyle::Underline { .. } => style.set_underline(),
            InlineStyle::Strike { .. } => style.set_strike(),
            InlineStyle::Fg { arg, .. } => style.set_foreground(*arg),
            InlineStyle::Bg { arg, .. } => style.set_background(*arg),
            InlineStyle::Fill { arg, .. } => style.set_fill(Some(*arg)),
            InlineStyle::Size { arg, .. } => style.set_size(*arg),
            InlineStyle::UserDefined { token, .. } => {
                inline_styles_to_style(page_styles.get(token).unwrap(), page_styles, style)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn font_style() {
        let page = r#"{(a italic) (b bold) (c mono)} ({a b c underline} "")"#;
        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.expressions[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
            Style {
                font_type: FontType::Mono,
                font_style: FontStyle {
                    italic: true,
                    bold: true,
                    underline: true,
                    ..FontStyle::default()
                },
                background: (0xff, 0xff, 0xff),
                foreground: (0x00, 0x00, 0x00),
                fill: None,
                size: 12,
            }
        );
    }

    #[test]
    fn bg_fg_application_order() {
        let page = r#"
        {(a (bg "b11111"))
         (b (fg "f22222") (bg "baaaad"))
         (c (bg "b33333"))}

        ({(fg "f11111") a (bg "b22222") b (fg "f33333") c} "")
        "#;

        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.expressions[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
            Style {
                font_type: FontType::Serif,
                font_style: FontStyle::default(),
                background: (0xb3, 0x33, 0x33),
                foreground: (0xf3, 0x33, 0x33),
                fill: None,
                size: 12,
            }
        );
    }

    #[test]
    fn font_type_application_order() {
        let page = r#"{(a sans) (b serif) (c mono)} ({a b c} "")"#;
        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.expressions[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
            Style {
                font_type: FontType::Mono,
                font_style: FontStyle::default(),
                background: (0xff, 0xff, 0xff),
                foreground: (0x00, 0x00, 0x00),
                fill: None,
                size: 12,
            }
        );
    }

    #[test]
    fn new_test_markup() {
        let train_doc = Document::from_page(
            &crate::markup::parse::parse(include_str!("../../../server/pages/new_test_markup.fml"))
                .unwrap(),
        );

        // im never doing this again
        let test_doc = Document {
            styles: HashMap::new(),
            expressions: vec![
                DocumentExpression {
                    style: Style {
                        font_style: FontStyle {
                            bold: true,
                            ..Default::default()
                        },
                        size: 32,
                        ..Style::new()
                    },
                    direction: Direction::Vertical,
                    contents: DocumentExpressionContents::Text {
                        text: String::from("Page title"),
                    },
                },
                DocumentExpression {
                    style: Style::new(),
                    direction: Direction::Vertical,
                    contents: DocumentExpressionContents::Text {
                        text: String::from("Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
                    },
                },
                DocumentExpression {
                    style: Style {
                        font_style: FontStyle {
                            bold: true,
                            ..Default::default()
                        },
                        ..Style::new()
                    },
                    direction: Direction::Vertical,
                    contents: DocumentExpressionContents::Text {
                        text: String::from("No gray or orange boxes should be visible below.")
                    },
                },
                DocumentExpression {
                    style: Style {
                        background: (245, 245, 245),
                        ..Style::new()
                    },
                    direction: Direction::Horizontal,
                    contents: DocumentExpressionContents::Children {
                        children: vec![
                            DocumentExpression {
                                style: Style {
                                    background: (218, 232, 252),
                                    ..Style::new()
                                },
                                direction: Direction::Vertical,
                                contents: DocumentExpressionContents::Text {
                                    text: String::from("Contrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature"),
                                },
                            },
                            DocumentExpression {
                                style: Style {
                                    background: (213, 232, 212),
                                    ..Style::new()
                                },
                                direction: Direction::Vertical,
                                contents: DocumentExpressionContents::Text {
                                    text: String::from("from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in"),
                                },
                            },
                            DocumentExpression {
                                style: Style {
                                    background: (255, 230, 204),
                                    ..Style::new()
                                },
                                direction: Direction::Vertical,
                                contents: DocumentExpressionContents::Children {
                                    children: vec![
                                        DocumentExpression {
                                            style: Style {
                                                background: (255, 242, 204),
                                                ..Style::new()
                                            },
                                            direction: Direction::Vertical,
                                            contents: DocumentExpressionContents::Text {
                                                text: String::from("Virginia, looked up one of the more obscure Latin")
                                            },
                                        },
                                        DocumentExpression {
                                            style: Style {
                                                background: (248, 206, 204),
                                                ..Style::new()
                                            },
                                            direction: Direction::Vertical,
                                            contents: DocumentExpressionContents::Link {
                                                text: String::from("from http://www.lipsum.com/"),
                                                url: String::from("http://www.lipsum.com/"),
                                            },
                                        },
                                    ],
                                },
                            },
                        ],
                    },
                },
                DocumentExpression {
                    style: Style::new(),
                    direction: Direction::Vertical,
                    contents: DocumentExpressionContents::Text {
                        text: String::from("But I must explain to you how all this mistaken idea of denouncing pleasure and praising pain was born and I will give you a complete account of the system, and expound the actual teachings of the great explorer of the truth, the master-builder of human happiness."),
                    },
                },
            ],
        };

        assert_eq!(train_doc, test_doc);
    }
}
