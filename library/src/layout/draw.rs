use crate::markup::scan::TokenKind;
use crate::markup::{ExpressionPayload, InlineStyle, PageExpressionAst, PageStyles};

// TODO yank all this out

use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontType {
    Mono,
    Serif,
    Sans,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct FontStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Style {
    pub font_type: FontType,
    pub font_style: FontStyle,
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub fill: Option<f32>,
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

// available width: 700px
// find out how many child box items there are, evenly distribute dependent on fill
// return the height that the content took up and draw the next item there

// in order to actually draw anything, we'll need to figure out bounding boxes for items.
// we'll do that here and then stick everything into DrawItems.

pub fn draw_item(
    item: &PageExpressionAst,
    page_styles: &PageStyles,
    start_point: (usize, usize),
    max_width: usize,
) -> usize {
    use super::DrawItem;
    println!("{:?} {:?} --- {:?}", start_point, max_width, item.builtin,);
    let mut style = Style::new();
    inline_styles_to_style(&item.styles, page_styles, &mut style);

    let dy = match &item.payload {
        ExpressionPayload::Children { children, .. } => match item.builtin.kind() {
            TokenKind::Wide => {
                let mut total_units = 0.0;
                let mut draw_items = Vec::new();
                for child in children {
                    let mut child_style = Style::new();
                    inline_styles_to_style(&child.styles, page_styles, &mut child_style);
                    total_units += child_style.fill.unwrap_or(0.0);
                    draw_items.push(DrawItem {
                        item: child,
                        style: child_style,
                    });
                }

                println!("total units: {}", total_units);
                let width_per_unit = max_width / total_units as usize;

                let mut current_x = 0;
                let mut largest_dy = 0;
                for child in draw_items {
                    let child_max_width = width_per_unit * child.style.fill.unwrap_or(0.0) as usize;
                    let dy = draw_item(
                        child.item,
                        page_styles,
                        (start_point.0 + current_x, start_point.1),
                        child_max_width,
                    );

                    println!(
                        "fill {:?} takes up {} px, should draw this item at relative x={} --- {:?}",
                        child.style.fill, child_max_width, current_x, child.item.builtin,
                    );

                    if dy > largest_dy {
                        largest_dy = dy;
                    }

                    current_x += child_max_width;
                }

                largest_dy
            }

            TokenKind::Inline => {
                // idk
                println!("inline");
                0
            }

            TokenKind::Tall => {
                // idk
                println!("vbox");
                1
            }

            _ => unreachable!("non-child-bearing item has children"),
        },

        ExpressionPayload::Text { .. } => {
            // wrap text
            println!("text");
            1
        }

        ExpressionPayload::Link { .. } => {
            println!("link");
            1
        }

        ExpressionPayload::Blob { .. } => {
            println!("blob");
            1
        }

        ExpressionPayload::Anchor { .. } => {
            println!("anchor");
            1
        }
    };

    println!("dy: {}", dy);
    dy
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
}
