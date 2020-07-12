//use crate::markup::scan::TokenKind;
use crate::markup::{InlineStyle, ItemPayload, PageItem, PageStyles};

use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontType {
    Mono,
    Serif,
    Sans,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FontStyle {
    Bold,
    Italic,
    Underline,
    Strike,
}

#[derive(Debug, PartialEq)]
pub struct Style {
    pub font_type: FontType,
    pub font_style: HashSet<FontStyle>,
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub fill: u8,
    pub size: usize,
}

#[derive(Default, Clone)]
struct StyleBuilder {
    font_type: Option<FontType>,
    font_style: Option<HashSet<FontStyle>>,
    background: Option<(u8, u8, u8)>,
    foreground: Option<(u8, u8, u8)>,
    fill: Option<u8>,
    size: Option<usize>,
}

#[allow(dead_code)]
impl StyleBuilder {
    fn build(self) -> Style {
        Style {
            font_type: self.font_type.unwrap_or_else(|| FontType::Serif),
            font_style: self.font_style.unwrap_or_else(|| HashSet::new()),
            background: self.background.unwrap_or_else(|| (0xff, 0xff, 0xff)),
            foreground: self.foreground.unwrap_or_else(|| (0x00, 0x00, 0x00)),
            fill: self.fill.unwrap_or_else(|| 1),
            size: self.size.unwrap_or_else(|| 12),
        }
    }

    fn font_type(mut self, font_type: FontType) -> Self {
        self.font_type = Some(font_type);
        self
    }

    fn font_style(mut self, font_style: HashSet<FontStyle>) -> Self {
        self.font_style = Some(font_style);
        self
    }

    fn background(mut self, background: (u8, u8, u8)) -> Self {
        self.background = Some(background);
        self
    }

    fn foreground(mut self, foreground: (u8, u8, u8)) -> Self {
        self.foreground = Some(foreground);
        self
    }

    fn fill(mut self, fill: u8) -> Self {
        self.fill = Some(fill);
        self
    }

    fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    fn set_font_type(&mut self, font_type: FontType) {
        self.font_type = Some(font_type);
    }

    fn add_font_style(&mut self, font_style: FontStyle) {
        match &mut self.font_style {
            Some(set) => {
                set.insert(font_style);
            }
            None => {
                let mut set = HashSet::new();
                set.insert(font_style);
                self.font_style = Some(set);
            }
        }
    }

    fn set_background(&mut self, background: (u8, u8, u8)) {
        self.background = Some(background);
    }

    fn set_foreground(&mut self, foreground: (u8, u8, u8)) {
        self.foreground = Some(foreground);
    }

    fn set_fill(&mut self, fill: u8) {
        self.fill = Some(fill);
    }

    fn set_size(&mut self, size: usize) {
        self.size = Some(size);
    }
}

fn inline_styles_to_style(
    styles: &[InlineStyle],
    page_styles: &PageStyles,
    builder: &mut StyleBuilder,
) {
    for style in styles {
        match style {
            InlineStyle::Mono { .. } => builder.set_font_type(FontType::Mono),
            InlineStyle::Serif { .. } => builder.set_font_type(FontType::Serif),
            InlineStyle::Sans { .. } => builder.set_font_type(FontType::Sans),
            InlineStyle::Bold { .. } => builder.add_font_style(FontStyle::Bold),
            InlineStyle::Italic { .. } => builder.add_font_style(FontStyle::Italic),
            InlineStyle::Underline { .. } => builder.add_font_style(FontStyle::Underline),
            InlineStyle::Strike { .. } => builder.add_font_style(FontStyle::Strike),
            InlineStyle::Fg { arg, .. } => builder.set_foreground(*arg),
            InlineStyle::Bg { arg, .. } => builder.set_background(*arg),
            InlineStyle::Fill { arg, .. } => builder.set_fill(*arg),
            InlineStyle::Size { arg, .. } => builder.set_size(*arg),
            InlineStyle::UserDefined { token, .. } => {
                inline_styles_to_style(page_styles.get(token).unwrap(), page_styles, builder)
            }
        }
    }
}

// available width: 700px
// find out how many child box items there are, evenly distribute dependent on fill
// return the height that the content took up and draw the next item there

#[derive(Clone, Copy, Debug, Default)]
pub struct Space {
    pub width: usize,
    pub height: Option<usize>,
}

pub fn draw_item(item: &PageItem, available_width: usize) -> Space {
    match &item.payload {
        ItemPayload::Children { children, .. } => {
            let per_child = available_width / children.len();

            Space {
                width: per_child,
                height: None,
            }
        }

        _ => Space {
            width: 0,
            height: None,
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn font_style() {
        let page = r#"{(a italic) (b bold) (c mono)} ({a b c underline} "")"#;
        let page = crate::parse_page(page).unwrap();
        let mut builder = StyleBuilder::default();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut builder);

        assert_eq!(
            builder.build(),
            Style {
                font_type: FontType::Mono,
                font_style: {
                    let mut set = HashSet::new();
                    set.insert(FontStyle::Italic);
                    set.insert(FontStyle::Bold);
                    set.insert(FontStyle::Underline);
                    set
                },
                background: (0xff, 0xff, 0xff),
                foreground: (0x00, 0x00, 0x00),
                fill: 1,
                size: 12,
            }
        );
    }

    #[test]
    fn background_foreground_application_order() {
        let page = r#"
        {(a (bg "b11111"))
         (b (fg "f22222") (bg "baaaad"))
         (c (bg "b33333"))}

        ({(fg "f11111") a (bg "b22222") b (fg "f33333") c} "")
        "#;

        let page = crate::parse_page(page).unwrap();
        let mut builder = StyleBuilder::default();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut builder);

        assert_eq!(
            builder.build(),
            Style {
                font_type: FontType::Serif,
                font_style: HashSet::new(),
                background: (0xb3, 0x33, 0x33),
                foreground: (0xf3, 0x33, 0x33),
                fill: 1,
                size: 12,
            }
        );
    }

    #[test]
    fn font_type_application_order() {
        let page = r#"{(a sans) (b serif) (c mono)} ({a b c} "")"#;
        let page = crate::parse_page(page).unwrap();
        let mut builder = StyleBuilder::default();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut builder);

        assert_eq!(
            builder.build(),
            Style {
                font_type: FontType::Mono,
                font_style: HashSet::new(),
                background: (0xff, 0xff, 0xff),
                foreground: (0x00, 0x00, 0x00),
                fill: 1,
                size: 12,
            }
        );
    }
}
