use super::color::*;
use super::font::*;
use super::style::*;

pub enum Item {
    Box {
        style: Option<Style>,
        children: Vec<Item>,
    },
    HBox {
        style: Option<Style>,
        children: Vec<Item>,
    },
    Text {
        style: Option<Style>,
        text: String,
    },
    Image(String),
}

impl Item {
    fn merge_styles(&self, _from: &Item) {
        todo!();
    }
}

pub struct Document {
    pub title: String,
    pub base_style: Style,
    pub tree: Vec<Item>,
}

// #0d162d
// #b2c6ff

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn color() {
        let cornsilk = Color::new(48, 14, 100);
        assert_eq!(cornsilk.to_rgb(), (0xff, 0xf8, 0xdb));

        let powder_blue = Color::from_rgb(0xb0, 0xe0, 0xe6);
        assert_eq!(powder_blue, Color::new(187, 23, 90));
        assert_eq!(powder_blue.to_rgb(), (0xb1, 0xdf, 0xe6));

        let dark_olive_green = Color::from_rgb(0x55, 0x6b, 0x2f);
        assert_eq!(dark_olive_green, Color::new(82, 56, 42));
        assert_eq!(dark_olive_green.to_rgb(), (0x55, 0x6b, 0x2f));

        let blue_violet1 = Color::from_rgb(0x8a, 0x2b, 0xe2);
        let (r, g, b) = blue_violet1.to_rgb();
        let blue_violet = Color::from_rgb(r, g, b);
        let (r, g, b) = blue_violet.to_rgb();
        let blue_violet = Color::from_rgb(r, g, b);
        let (r, g, b) = blue_violet.to_rgb();
        let blue_violet = Color::from_rgb(r, g, b);
        let (r, g, b) = blue_violet.to_rgb();
        let blue_violet = Color::from_rgb(r, g, b);
        let (r, g, b) = blue_violet.to_rgb();
        let blue_violet = Color::from_rgb(r, g, b);
        assert_eq!(blue_violet, blue_violet1);
        assert_eq!(blue_violet.to_rgb(), (0x8a, 0x2b, 0xe3));
    }

    #[test]
    #[rustfmt::skip]
    fn style() {
        let base_font = FontBuilder::new().monospace().italic().size(32).build();
        let base = StyleBuilder::new()
            .font_properties(base_font.clone())
            .foreground(Color::from_rgb(0xba, 0xb1, 0xb3))
            .build();
        let sub = StyleBuilder::with(base.clone())
            .font_properties(
                FontBuilder::with(base_font.clone())
                    .sans()
                    .underline()
                    .build(),
            )
            .foreground(Color::from_rgb(0x33, 0x43, 0xaa))
            .build();
        let sub2 = StyleBuilder::with(sub.clone())
            .foreground(Color::white())
            .background(Color::black())
            .build();

        assert_eq!(base.font_properties().font_type(), &FontType::Monospace);
        assert_eq!(base.font_properties().font_style(), &[FontStyle::Italic]);
        assert_eq!(sub.font_properties().font_type(), &FontType::Sans);
        assert_eq!(sub.font_properties().font_style(), &[FontStyle::Italic, FontStyle::Underline]);
        assert_eq!(sub.foreground(), &Color::from_rgb(0x33, 0x43, 0xaa));
        assert_eq!(sub2.foreground(), &Color::white());
        assert_eq!(sub2.background(), &Color::black());
        assert_eq!(sub2.font_properties().font_style(), &[FontStyle::Italic, FontStyle::Underline]);
    }

    #[test]
    fn sample() {
        let base_style = Style::default();
        let quote_box = StyleBuilder::with(base_style.clone())
            .background(Color::from_rgb(0xff, 0xf8, 0xdc))
            .foreground(Color::from_rgb(0x30, 0x30, 0x30))
            .build();
        let quote_text = StyleBuilder::with(base_style.clone())
            .background(Color::from_rgb(0xff, 0xf8, 0xdc))
            .foreground(Color::from_rgb(0x60, 0x60, 0x60))
            .build();
        let footnote = StyleBuilder::with(base_style.clone())
            .foreground(Color::from_rgb(0x75, 0x75, 0x75))
            .font_properties(FontBuilder::new().italic().sans().build())
            .build();

        let document = Document {
            title: String::from("Page title"),
            base_style: base_style.clone(),
            tree: vec![
                Item::HBox {
                    style: None,
                    children: vec![
                        Item::Text {
                            style: Some(Style::header(1)),
                            text: String::from("Lorem ipsum example"),
                        },
                        Item::Image(String::from("header.jpg")),
                    ],
                },
                Item::Text {
                    style: None,
                    text: String::from("Lorem ipsum dolor sit amet, consectetur"),
                },
                Item::HBox {
                    style: Some(quote_box.clone()),
                    children: vec![
                        Item::Text {
                            style: Some(quote_text.clone()),
                            text: String::from("Contrary to popular belief, Lorem Ipsum"),
                        },
                        Item::Text {
                            style: Some(quote_text.clone()),
                            text: String::from("from 45 BC, making it over 2000 years old."),
                        },
                        Item::Box {
                            style: Some(quote_box.clone()),
                            children: vec![
                                Item::Text {
                                    style: Some(quote_text.clone()),
                                    text: String::from("looked up one of the more obscure Latin"),
                                },
                                Item::Text {
                                    style: Some(
                                        StyleBuilder::with(footnote.clone()).height(20).build(),
                                    ),
                                    text: String::from("from https://www.lipsum.com/"),
                                },
                            ],
                        },
                    ],
                },
            ],
        };
    }
}
