use crate::markup::scan::{Token, TokenKind};
use crate::markup::{InlineStyle, ItemPayload, Page, PageItem};

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct OwnedToken {
    kind: TokenKind,
    line: usize,
    lexeme: String,
}

impl PartialEq for OwnedToken {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexeme == other.lexeme
    }
}

impl Eq for OwnedToken {}

impl std::hash::Hash for OwnedToken {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.kind.hash(hasher);
        self.lexeme.hash(hasher);
    }
}

#[derive(Clone, Debug)]
pub struct OwnedPage {
    pub page_styles: OwnedPageStyles,
    pub items: Vec<OwnedPageItem>,
}

pub type OwnedPageStyles = HashMap<OwnedToken, Vec<OwnedInlineStyle>>;

#[derive(Debug, PartialEq, Clone)]
pub struct OwnedPageItem {
    pub builtin: OwnedToken,
    pub inline_styles: Vec<OwnedInlineStyle>,
    pub payload: OwnedItemPayload,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OwnedItemPayload {
    Text {
        text: Vec<OwnedToken>,
    },
    Children {
        children: Vec<OwnedPageItem>,
        line: usize,
    },
    Link {
        link: OwnedToken,
        text: Vec<OwnedToken>,
    },
    Blob {
        name: OwnedToken,
        alt: Vec<OwnedToken>,
    },
    Anchor {
        anchor: OwnedToken,
    },
}

#[rustfmt::skip]
#[derive(Debug, PartialEq, Clone)]
pub enum OwnedInlineStyle {
    Mono { token: OwnedToken },
    Serif { token: OwnedToken },
    Sans { token: OwnedToken },
    Bold { token: OwnedToken },
    Italic { token: OwnedToken },
    Underline { token: OwnedToken },
    Strike { token: OwnedToken },
    Fg { token: OwnedToken, arg: (u8, u8, u8) },
    Bg { token: OwnedToken, arg: (u8, u8, u8) },
    Fill { token: OwnedToken, arg: u8 },
    Size { token: OwnedToken, arg: usize },
    UserDefined { token: OwnedToken },
}

impl Data for OwnedPage {
    fn same(&self, _: &Self) -> bool {
        // ???
        false
    }
}

impl From<&Token<'_>> for OwnedToken {
    fn from(token: &Token<'_>) -> Self {
        OwnedToken {
            kind: token.kind(),
            line: token.line(),
            lexeme: token.clone_lexeme(),
        }
    }
}

#[rustfmt::skip]
impl From<&InlineStyle<'_>> for OwnedInlineStyle {
    fn from(inline_style: &InlineStyle<'_>) -> Self {
        match inline_style {
            InlineStyle::Mono { token }
                => OwnedInlineStyle::Mono { token: token.into() },
            InlineStyle::Serif { token }
                => OwnedInlineStyle::Serif { token: token.into() },
            InlineStyle::Sans { token }
                => OwnedInlineStyle::Sans { token: token.into() },
            InlineStyle::Bold { token }
                => OwnedInlineStyle::Bold { token: token.into() },
            InlineStyle::Italic { token }
                => OwnedInlineStyle::Italic { token: token.into() },
            InlineStyle::Underline { token }
                => OwnedInlineStyle::Underline { token: token.into() },
            InlineStyle::Strike { token }
                => OwnedInlineStyle::Strike { token: token.into() },
            InlineStyle::Fg { token, arg }
                => OwnedInlineStyle::Fg { token: token.into(), arg: *arg },
            InlineStyle::Bg { token, arg }
                => OwnedInlineStyle::Bg { token: token.into(), arg: *arg },
            InlineStyle::Fill { token, arg }
                => OwnedInlineStyle::Fill { token: token.into(), arg: *arg },
            InlineStyle::Size { token, arg }
                => OwnedInlineStyle::Size { token: token.into(), arg: *arg },
            InlineStyle::UserDefined { token }
                => OwnedInlineStyle::UserDefined { token: token.into() },
        }
    }
}

impl From<&PageItem<'_>> for OwnedPageItem {
    fn from(item: &PageItem) -> Self {
        let builtin = (&item.builtin).into();
        let inline_styles = item.inline_styles.iter().map(From::from).collect();
        let payload = match &item.payload {
            ItemPayload::Text { text } => OwnedItemPayload::Text {
                text: text.iter().map(From::from).collect(),
            },
            ItemPayload::Children { children, line } => OwnedItemPayload::Children {
                children: children.iter().map(From::from).collect(),
                line: *line,
            },
            ItemPayload::Link { link, text } => OwnedItemPayload::Link {
                link: link.into(),
                text: text.iter().map(From::from).collect(),
            },
            ItemPayload::Blob { name, alt } => OwnedItemPayload::Blob {
                name: name.into(),
                alt: alt.iter().map(From::from).collect(),
            },
            ItemPayload::Anchor { anchor } => OwnedItemPayload::Anchor {
                anchor: anchor.into(),
            },
        };

        OwnedPageItem {
            builtin,
            inline_styles,
            payload,
        }
    }
}

impl From<Page<'_>> for OwnedPage {
    fn from(page: Page<'_>) -> Self {
        let mut page_styles = HashMap::new();
        for (selector, styles) in &page.page_styles {
            page_styles.insert(selector.into(), styles.iter().map(From::from).collect());
        }

        let mut items = Vec::new();
        for item in &page.items {
            items.push(item.into());
        }

        OwnedPage { page_styles, items }
    }
}

pub struct PageWidget;

impl Widget<OwnedPage> for PageWidget {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut OwnedPage, _: &Env) {
        //println!("{:?}", event);
    }

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &OwnedPage, _: &Env) {
        //println!("{:?}", lifecycle);
    }

    fn update(&mut self, _: &mut UpdateCtx, _: &OwnedPage, _: &OwnedPage, _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &OwnedPage, _: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, painter: &mut PaintCtx, page: &OwnedPage, _: &Env) {
        use crate::druid::piet::{FontBuilder, Text, TextLayoutBuilder};
        use crate::druid::{Color, RenderContext};

        let mut i = 24.;
        for item in &page.items {
            let text = match &item.payload {
                OwnedItemPayload::Text { text } => text
                    .iter()
                    .fold(String::new(), |acc, next| format!("{}{}", acc, next.lexeme)),
                OwnedItemPayload::Link { link, text } => format!(
                    "link {}: {}",
                    link.lexeme,
                    text.iter()
                        .fold(String::new(), |acc, next| format!("{}{}", acc, next.lexeme))
                ),
                _ => String::from("dingus"),
            };

            let font = painter
                .text()
                .new_font_by_name("Segoe UI", 24.0)
                .build()
                .unwrap();

            let layout = painter
                .text()
                .new_text_layout(&font, &text, std::f64::INFINITY)
                .build()
                .unwrap();

            painter.draw_text(&layout, (10., i), &Color::rgba8(0x00, 0x00, 0x00, 0xff));
            i += 24.;
        }
    }
}
