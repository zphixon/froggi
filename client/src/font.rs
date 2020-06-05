#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FontStyle {
    Strikethrough,
    Bold,
    Italic,
    Underline,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FontType {
    Monospace,
    Serif,
    Sans,
}

#[derive(Clone, Debug)]
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
    pub fn font_style(&self) -> &Vec<FontStyle> {
        &self.font_style
    }

    pub fn font_type(&self) -> &FontType {
        &self.font_type
    }

    pub fn size(&self) -> &u8 {
        &self.size
    }
}

pub struct FontBuilder {
    base: Option<FontProperties>,
    font_style: Vec<FontStyle>,
    font_type: Option<FontType>,
    size: Option<u8>,
}

impl FontBuilder {
    pub fn new() -> Self {
        Self {
            base: None,
            font_style: Vec::new(),
            font_type: None,
            size: None,
        }
    }

    pub fn with(base: FontProperties) -> Self {
        Self {
            font_style: base.font_style.clone(),
            base: Some(base),
            ..Self::new()
        }
    }

    pub fn build(self) -> FontProperties {
        let default = self.base.unwrap_or(FontProperties::default());
        FontProperties {
            font_style: self.font_style,
            font_type: self.font_type.unwrap_or(default.font_type),
            size: self.size.unwrap_or(default.size),
        }
    }

    pub fn serif(self) -> Self {
        Self {
            font_type: Some(FontType::Serif),
            ..self
        }
    }

    pub fn sans(self) -> Self {
        Self {
            font_type: Some(FontType::Sans),
            ..self
        }
    }

    pub fn monospace(self) -> Self {
        Self {
            font_type: Some(FontType::Monospace),
            ..self
        }
    }

    pub fn strikethrough(mut self) -> Self {
        self.font_style.push(FontStyle::Strikethrough);
        self
    }

    pub fn bold(mut self) -> Self {
        self.font_style.push(FontStyle::Bold);
        self
    }

    pub fn italic(mut self) -> Self {
        self.font_style.push(FontStyle::Italic);
        self
    }

    pub fn underline(mut self) -> Self {
        self.font_style.push(FontStyle::Underline);
        self
    }

    pub fn size(mut self, size: u8) -> Self {
        self.size = Some(size);
        self
    }
}
