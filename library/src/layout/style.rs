use super::color::Color;
use super::font::FontProperties;

use crate::markup::{InlineStyle, PageStyle, WithArg, WithoutArg};
use crate::{FroggiError, MarkupError};

/// Style on a FML item.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    selector: Option<String>,
    fill: Option<u8>,
    foreground: Color,
    background: Color,
    font_properties: FontProperties,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            selector: None,
            fill: None,
            foreground: Color::black(),
            background: Color::white(),
            font_properties: FontProperties::default(),
        }
    }
}

impl Style {
    pub fn from_page_style(page_style: PageStyle) -> Result<Self, FroggiError> {
        let mut style = Style::default();

        for inline_style in page_style.styles {
            // page styles are always builtin
            let builtin_style = get_by_name(&inline_style).ok_or_else(|| {
                FroggiError::markup(
                    MarkupError::UnknownStyle {
                        style: inline_style.token().clone_lexeme(),
                    },
                    inline_style.token().line(),
                )
            })?;

            let has_arg = inline_style.has_arg();
            let takes_arg = builtin_style.takes_arg();

            match builtin_style.arg_kind {
                ArgKind::Color(handler)
                | ArgKind::Percent(handler)
                | ArgKind::FontSize(handler)
                    if has_arg && takes_arg =>
                {
                    handler(&mut style, inline_style.as_with_arg())?;
                }

                ArgKind::None(handler) if !has_arg && !takes_arg => {
                    handler(&mut style, inline_style.as_without_arg())?;
                }

                _ => {
                    if takes_arg {
                        return Err(FroggiError::markup(
                            MarkupError::RequiresArgument {
                                style: inline_style.token().clone_lexeme(),
                            },
                            inline_style.token().line(),
                        ));
                    } else {
                        return Err(FroggiError::markup(
                            MarkupError::NoArgumentAllowed {
                                style: inline_style.token().clone_lexeme(),
                            },
                            inline_style.token().line(),
                        ));
                    }
                }
            }
        }

        style.set_selector(page_style.selector.clone_lexeme());

        Ok(style)
    }

    fn ignore_with(&mut self, _with_arg: &WithArg) -> Result<(), FroggiError> {
        Ok(())
    }

    fn ignore_without(&mut self, _without_arg: &WithoutArg) -> Result<(), FroggiError> {
        Ok(())
    }

    fn set_selector(&mut self, selector: String) {
        self.selector = Some(selector);
    }

    fn set_foreground_color(&mut self, fg: &WithArg) -> Result<(), FroggiError> {
        self.foreground = Color::from_token(&fg.arg)?;
        Ok(())
    }

    fn set_background_color(&mut self, bg: &WithArg) -> Result<(), FroggiError> {
        self.background = Color::from_token(&bg.arg)?;
        Ok(())
    }

    fn set_fill(&mut self, fill: &WithArg) -> Result<(), FroggiError> {
        let percent = fill.arg.trimmed_lexeme();
        let percentage = match percent.parse::<u8>() {
            Ok(percent) => percent,
            Err(_) => {
                return Err(FroggiError::markup(
                    MarkupError::IncorrectPercent { percent },
                    fill.arg.line(),
                ))
            }
        };

        if percentage > 100 {
            return Err(FroggiError::markup(
                MarkupError::IncorrectPercent { percent },
                fill.arg.line(),
            ));
        }

        self.fill = Some(percentage);

        Ok(())
    }
}

type WithArgAdder = fn(&mut Style, &WithArg) -> Result<(), FroggiError>;
type WithoutArgAdder = fn(&mut Style, &WithoutArg) -> Result<(), FroggiError>;

pub enum ArgKind {
    Color(WithArgAdder),
    Percent(WithArgAdder),
    FontSize(WithArgAdder),
    None(WithoutArgAdder),
}

impl std::fmt::Debug for ArgKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArgKind::Color(_) => write!(f, "Color"),
            ArgKind::Percent(_) => write!(f, "Percent"),
            ArgKind::FontSize(_) => write!(f, "FontSize"),
            ArgKind::None(_) => write!(f, "None"),
        }
    }
}

#[derive(Debug)]
pub struct BuiltinStyle {
    name: &'static str,
    is_font_property: bool,
    arg_kind: ArgKind,
}

impl BuiltinStyle {
    fn takes_arg(&self) -> bool {
        match self.arg_kind {
            ArgKind::None(_) => false,
            _ => true,
        }
    }
}

// TODO: something like this https://gist.github.com/zphixon/386b86d3ca472e9c8a6cf556c0efadc9
// TODO: probably make this a lazy static hashmap instead
#[rustfmt::skip]
pub const BUILTIN_STYLES: &[BuiltinStyle] = &[
    BuiltinStyle { name: "fg",            is_font_property: false,      arg_kind: ArgKind::Color(Style::set_foreground_color),        },
    BuiltinStyle { name: "bg",            is_font_property: false,      arg_kind: ArgKind::Color(Style::set_background_color),        },
    BuiltinStyle { name: "fill",          is_font_property: false,      arg_kind: ArgKind::Percent(Style::set_fill),      },
    BuiltinStyle { name: "size",          is_font_property: true,       arg_kind: ArgKind::FontSize(Style::ignore_with),     },
    BuiltinStyle { name: "monospace",     is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "serif",         is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "sans",          is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "strike",        is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "bold",          is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "italic",        is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
    BuiltinStyle { name: "underline",     is_font_property: true,       arg_kind: ArgKind::None(Style::ignore_without),      },
];

fn get_by_name(inline_style: &InlineStyle) -> Option<&'static BuiltinStyle> {
    BUILTIN_STYLES
        .iter()
        .find(|style| style.name == inline_style.name())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::markup::scan::{Token, TokenKind};
    use crate::markup::WithArg;

    #[test]
    fn check_if_size_is_font_property() {
        assert!(
            get_by_name(&InlineStyle::WithArg(WithArg {
                name: Token::new(TokenKind::Identifier, 1, "size"),
                arg: Token::new(TokenKind::Text, 1, "\"32\"")
            }))
            .unwrap()
            .is_font_property
        );
    }

    #[test]
    fn fg_color() {
        let inline_style = r#"({(fg "feed10") item} "text ")"#;
        let page = crate::markup::parse::parse(inline_style).unwrap();
        let inline_style = &page.items[0].inline_styles[0];
        let mut style = Style::default();
        if let ArgKind::Color(handler) = get_by_name(inline_style).unwrap().arg_kind {
            handler(&mut style, inline_style.as_with_arg()).unwrap();
            assert_eq!(style.foreground, Color::new(0xfe, 0xed, 0x10));
        } else {
            panic!();
        }
    }

    #[test]
    fn from_page_style() {
        let page_style = PageStyle {
            selector: Token::new(TokenKind::Identifier, 1, "name"),
            styles: vec![
                InlineStyle::WithArg(WithArg {
                    name: Token::new(TokenKind::Identifier, 1, "fg"),
                    arg: Token::new(TokenKind::Text, 1, "\"dedb1f\""),
                }),
                InlineStyle::WithoutArg(WithoutArg {
                    name: Token::new(TokenKind::Identifier, 1, "sans"),
                }),
            ],
        };

        let style = Style::from_page_style(page_style).unwrap();
        assert_eq!(
            style,
            Style {
                selector: Some("name".into()),
                fill: None,
                foreground: Color::new(0xde, 0xdb, 0x1f),
                background: Color::white(),
                font_properties: FontProperties::default(),
            }
        );
    }
}
