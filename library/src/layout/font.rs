use std::collections::HashSet;

/// Style of a font.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
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
#[derive(PartialEq, Clone, Debug)]
pub struct FontProperties {
    pub font_style: HashSet<FontStyle>,
    pub font_type: FontType,
    pub size: u8,
}

impl Default for FontProperties {
    fn default() -> Self {
        Self {
            font_style: HashSet::new(),
            font_type: FontType::Serif,
            size: 12,
        }
    }
}

impl FontProperties {
    pub fn font_style(&self) -> &HashSet<FontStyle> {
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
        self.font_style.insert(font_style);
    }

    pub fn set_size(&mut self, size: u8) {
        self.size = size;
    }
}
