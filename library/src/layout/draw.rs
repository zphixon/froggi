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

impl Style {
    fn new() -> Style {
        Style {
            font_type: FontType::Serif,
            font_style: HashSet::new(),
            background: (0xff, 0xff, 0xff),
            foreground: (0x00, 0x00, 0x00),
            fill: 1,
            size: 12,
        }
    }

    fn set_font_type(&mut self, font_type: FontType) {
        self.font_type = font_type;
    }

    fn add_font_style(&mut self, font_style: FontStyle) {
        self.font_style.insert(font_style);
    }

    fn set_background(&mut self, background: (u8, u8, u8)) {
        self.background = background;
    }

    fn set_foreground(&mut self, foreground: (u8, u8, u8)) {
        self.foreground = foreground;
    }

    fn set_fill(&mut self, fill: u8) {
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
            InlineStyle::Bold { .. } => style.add_font_style(FontStyle::Bold),
            InlineStyle::Italic { .. } => style.add_font_style(FontStyle::Italic),
            InlineStyle::Underline { .. } => style.add_font_style(FontStyle::Underline),
            InlineStyle::Strike { .. } => style.add_font_style(FontStyle::Strike),
            InlineStyle::Fg { arg, .. } => style.set_foreground(*arg),
            InlineStyle::Bg { arg, .. } => style.set_background(*arg),
            InlineStyle::Fill { arg, .. } => style.set_fill(*arg),
            InlineStyle::Size { arg, .. } => style.set_size(*arg),
            InlineStyle::UserDefined { token, .. } => {
                inline_styles_to_style(page_styles.get(token).unwrap(), page_styles, style)
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

pub fn draw_item(item: &PageItem, page_styles: &PageStyles, available_width: usize) -> Space {
    let mut style = Style::new();
    inline_styles_to_style(&item.styles, page_styles, &mut style);

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
        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
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
    fn bg_fg_application_order() {
        let page = r#"
        {(a (bg "b11111"))
         (b (fg "f22222") (bg "baaaad"))
         (c (bg "b33333"))}

        ({(fg "f11111") a (bg "b22222") b (fg "f33333") c} "")
        "#;

        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
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
        let page = crate::markup::parse::parse(page).unwrap();
        let mut style = Style::new();
        inline_styles_to_style(&page.items[0].styles, &page.styles, &mut style);

        assert_eq!(
            style,
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
