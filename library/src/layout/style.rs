use super::color::Color;
use super::font::{FontProperties, FontStyle, FontType};

use crate::markup::{InlineStyle, PageStyle, WithArg, WithoutArg};
use crate::{AddMsg, FroggiError, MarkupError};

/// Style on a FML item.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    fill: Option<u8>,
    foreground: Color,
    background: Color,
    font_properties: FontProperties,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: None,
            foreground: Color::black(),
            background: Color::white(),
            font_properties: FontProperties::default(),
        }
    }
}

impl Style {
    pub fn from_inline_styles(inline_styles: &[InlineStyle]) -> Result<Self, FroggiError> {
        let mut style = Style::default();
        for inline_style in inline_styles {
            let builtin_style = get_by_name(&inline_style)
                .ok_or_else(|| {
                    FroggiError::markup(
                        MarkupError::UnknownStyle {
                            style: inline_style.token().clone_lexeme(),
                        },
                        inline_style.token().line(),
                    )
                })
                .msg_str("page styles must only use built-in styles")?;

            let has_arg = inline_style.has_arg();
            let takes_arg = builtin_style.takes_arg();

            match builtin_style.arg {
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

        Ok(style)
    }

    pub fn from_page_style(page_style: &PageStyle) -> Result<Self, FroggiError> {
        let mut style = Style::from_inline_styles(&page_style.styles)?;
        Ok(style)
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
        let percent = fill.arg.lexeme();
        let percentage = match percent.parse::<u8>() {
            Ok(percent) => percent,
            Err(_) => {
                return Err(FroggiError::markup(
                    MarkupError::IncorrectPercent {
                        percent: percent.to_owned(),
                    },
                    fill.arg.line(),
                ))
            }
        };

        if percentage > 100 {
            return Err(FroggiError::markup(
                MarkupError::IncorrectPercent {
                    percent: percent.to_owned(),
                },
                fill.arg.line(),
            ));
        }

        self.fill = Some(percentage);

        Ok(())
    }

    fn set_font_size(&mut self, size: &WithArg) -> Result<(), FroggiError> {
        let num = size.arg.lexeme();
        let size = num.parse::<u8>().map_err(|_| {
            FroggiError::markup(
                MarkupError::IncorrectNumber {
                    num: num.to_owned(),
                },
                size.arg.line(),
            )
        })?;
        self.font_properties.set_size(size);
        Ok(())
    }

    fn set_font_monospace(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.set_type(FontType::Monospace);
        Ok(())
    }

    fn set_font_serif(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.set_type(FontType::Serif);
        Ok(())
    }

    fn set_font_sans(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.set_type(FontType::Sans);
        Ok(())
    }

    fn set_font_strike(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.add_style(FontStyle::Strike);
        Ok(())
    }

    fn set_font_bold(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.add_style(FontStyle::Bold);
        Ok(())
    }

    fn set_font_italic(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.add_style(FontStyle::Italic);
        Ok(())
    }

    fn set_font_underline(&mut self, _: &WithoutArg) -> Result<(), FroggiError> {
        self.font_properties.add_style(FontStyle::Underline);
        Ok(())
    }
}

type WithArgAdder = fn(&mut Style, &WithArg) -> Result<(), FroggiError>;
type WithoutArgAdder = fn(&mut Style, &WithoutArg) -> Result<(), FroggiError>;

#[derive(Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct BuiltinStyle {
    font: bool,
    arg: ArgKind,
}

impl BuiltinStyle {
    fn takes_arg(&self) -> bool {
        match self.arg {
            ArgKind::None(_) => false,
            _ => true,
        }
    }
}

// TODO: something like this https://gist.github.com/zphixon/386b86d3ca472e9c8a6cf556c0efadc9
use std::collections::HashMap;
lazy_static::lazy_static! {
    static ref BUILTIN_STYLES: HashMap<&'static str, BuiltinStyle> = {
        let mut builtin_styles = HashMap::new();
        builtin_styles.insert("fg",            BuiltinStyle { font: false, arg: ArgKind::Color(Style::set_foreground_color), });
        builtin_styles.insert("bg",            BuiltinStyle { font: false, arg: ArgKind::Color(Style::set_background_color), });
        builtin_styles.insert("fill",          BuiltinStyle { font: false, arg: ArgKind::Percent(Style::set_fill),           });
        builtin_styles.insert("size",          BuiltinStyle { font: true,  arg: ArgKind::FontSize(Style::set_font_size),     });
        builtin_styles.insert("mono",          BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_monospace),    });
        builtin_styles.insert("serif",         BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_serif),        });
        builtin_styles.insert("sans",          BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_sans),         });
        builtin_styles.insert("strike",        BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_strike),       });
        builtin_styles.insert("bold",          BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_bold),         });
        builtin_styles.insert("italic",        BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_italic),       });
        builtin_styles.insert("underline",     BuiltinStyle { font: true,  arg: ArgKind::None(Style::set_font_underline),    });
        builtin_styles
    };
}

fn get_by_name(style: &InlineStyle) -> Option<BuiltinStyle> {
    BUILTIN_STYLES
        .get(style.name())
        .map(|builtin_style| *builtin_style)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::markup::scan::{Token, TokenKind};
    use crate::markup::WithArg;

    #[test]
    fn non_builtin_page_style() {
        assert!(Style::from_page_style(&PageStyle {
            selector: Token::new(TokenKind::Identifier, 1, "test"),
            styles: vec![InlineStyle::WithoutArg(WithoutArg {
                name: Token::new(TokenKind::Identifier, 1, "not-built-in"),
            })],
        })
        .is_err());
    }

    #[test]
    fn check_if_size_is_font_property() {
        assert!(
            get_by_name(&InlineStyle::WithArg(WithArg {
                name: Token::new(TokenKind::Identifier, 1, "size"),
                arg: Token::new(TokenKind::Text, 1, r#""32""#)
            }))
            .unwrap()
            .font
        );
    }

    #[test]
    fn fg_color() {
        let inline_style = r#"({(fg "feed10") item} "text ")"#;
        let page = crate::markup::parse::parse(inline_style).unwrap();
        let inline_style = &page.items[0].inline_styles[0];
        let mut style = Style::default();
        if let ArgKind::Color(handler) = get_by_name(inline_style).unwrap().arg {
            handler(&mut style, inline_style.as_with_arg()).unwrap();
            assert_eq!(style.foreground, Color::new(0xfe, 0xed, 0x10));
        } else {
            panic!();
        }
        assert!(get_by_name(&page.items[0].inline_styles[1]).is_none());
    }

    #[test]
    fn from_page_style() {
        let page_style = PageStyle {
            selector: Token::new(TokenKind::Identifier, 1, "name"),
            styles: vec![
                InlineStyle::WithArg(WithArg {
                    name: Token::new(TokenKind::Identifier, 1, "fg"),
                    arg: Token::new(TokenKind::String, 1, r#""dedb1f""#),
                }),
                InlineStyle::WithoutArg(WithoutArg {
                    name: Token::new(TokenKind::Identifier, 1, "sans"),
                }),
            ],
        };

        let style = Style::from_page_style(&page_style).unwrap();
        assert_eq!(
            style,
            Style {
                fill: None,
                foreground: Color::new(0xde, 0xdb, 0x1f),
                background: Color::white(),
                font_properties: FontProperties {
                    font_type: FontType::Sans,
                    ..FontProperties::default()
                }
            }
        );
    }

    #[test]
    fn parse_from_page_style() {
        let page = r#"{(text serif (fg "303030"))
                       (quote-box (bg "fff8dc"))
                       (quote-text sans (fg "606060"))
                       (footnote (fg "757575") sans italic)}"#;
        let page = crate::markup::parse::parse(page).unwrap();
        let styles: Vec<Style> = page
            .page_styles
            .iter()
            .map(|page_style| Style::from_page_style(page_style).unwrap())
            .collect();

        let mut hm = std::collections::HashSet::new();
        hm.insert(FontStyle::Italic);
        assert_eq!(
            styles,
            vec![
                Style {
                    fill: None,
                    foreground: Color::new(0x30, 0x30, 0x30),
                    background: Color::white(),
                    font_properties: FontProperties::default(),
                },
                Style {
                    fill: None,
                    foreground: Color::black(),
                    background: Color::new(0xff, 0xf8, 0xdc),
                    font_properties: FontProperties::default(),
                },
                Style {
                    fill: None,
                    foreground: Color::new(0x60, 0x60, 0x60),
                    background: Color::white(),
                    font_properties: FontProperties {
                        font_type: FontType::Sans,
                        ..FontProperties::default()
                    },
                },
                Style {
                    fill: None,
                    foreground: Color::new(0x75, 0x75, 0x75),
                    background: Color::white(),
                    font_properties: FontProperties {
                        font_style: hm,
                        font_type: FontType::Sans,
                        ..FontProperties::default()
                    },
                },
            ]
        );
    }
}
