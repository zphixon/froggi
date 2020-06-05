use super::color::*;
use super::font::*;

#[derive(Clone, Debug)]
pub struct Style {
    width: Option<u8>,
    height: Option<u8>,
    foreground: Color,
    background: Color,
    font_properties: FontProperties,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            foreground: Color::black(),
            background: Color::white(),
            font_properties: FontProperties::default(),
        }
    }
}

impl Style {
    pub fn header(num: u8) -> Self {
        let size = match num {
            1 => 48,
            2 => 36,
            3 => 28,
            4 => 26,
            5 => 24,
            6 => 22,
            7 => 20,
            9 => 18,
            _ => 14,
        };

        StyleBuilder::new()
            .font_properties(FontBuilder::new().bold().size(size).build())
            .build()
    }

    pub fn width(&self) -> &Option<u8> {
        &self.width
    }

    pub fn height(&self) -> &Option<u8> {
        &self.height
    }

    pub fn foreground(&self) -> &Color {
        &self.foreground
    }

    pub fn background(&self) -> &Color {
        &self.background
    }

    pub fn font_properties(&self) -> &FontProperties {
        &self.font_properties
    }
}

pub struct StyleBuilder {
    base: Option<Style>,
    width: Option<u8>,
    height: Option<u8>,
    foreground: Option<Color>,
    background: Option<Color>,
    font_properties: Option<FontProperties>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            base: None,
            width: None,
            height: None,
            foreground: None,
            background: None,
            font_properties: None,
        }
    }

    pub fn with(base: Style) -> Self {
        Self {
            base: Some(base),
            ..Self::new()
        }
    }

    pub fn build(self) -> Style {
        let default = self.base.unwrap_or(Style::default());
        Style {
            width: self.width,
            height: self.height,
            foreground: self.foreground.unwrap_or(default.foreground),
            background: self.background.unwrap_or(default.background),
            font_properties: self.font_properties.unwrap_or(default.font_properties),
        }
    }

    pub fn width(self, width: u8) -> Self {
        Self {
            width: Some(width),
            ..self
        }
    }

    pub fn height(self, height: u8) -> Self {
        Self {
            height: Some(height),
            ..self
        }
    }

    pub fn foreground(self, foreground: Color) -> Self {
        Self {
            foreground: Some(foreground),
            ..self
        }
    }

    pub fn background(self, background: Color) -> Self {
        Self {
            background: Some(background),
            ..self
        }
    }

    pub fn font_properties(self, font_properties: FontProperties) -> Self {
        Self {
            font_properties: Some(font_properties),
            ..self
        }
    }
}
