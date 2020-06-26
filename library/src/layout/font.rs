/// Style of a font.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FontStyle {
    Strike,
    Bold,
    Italic,
    Underline,
}

/// Type of font.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FontType {
    Monospace,
    Serif,
    Sans,
}

/// Properties of a font.
#[derive(Clone, Debug, PartialEq)]
pub struct FontProperties {
    font_style: Vec<FontStyle>,
    font_type: FontType,
    size: u8,
}

impl Default for FontProperties {
    fn default() -> Self {
        Self {
            font_style: vec![],
            font_type: FontType::Serif,
            size: 12,
        }
    }
}

impl FontProperties {
    pub fn sans() -> Self {
        FontProperties {
            font_type: FontType::Sans,
            ..Default::default()
        }
    }

    pub fn font_style(&self) -> &Vec<FontStyle> {
        &self.font_style
    }

    pub fn font_type(&self) -> &FontType {
        &self.font_type
    }

    pub fn size(&self) -> &u8 {
        &self.size
    }

    pub fn set_type(&mut self, font_type: FontType) {
        self.font_type = font_type;
    }

    pub fn add_style(&mut self, font_style: FontStyle) {
        self.font_style.push(font_style);
    }

    pub fn set_size(&mut self, size: u8) {
        self.size = size;
    }
}
