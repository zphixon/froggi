use crate::{FroggiError, MarkupError};

/// HSV color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, b: u8, g: u8) -> Self {
        Color { r, g, b }
    }

    pub fn white() -> Self {
        Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }

    pub fn black() -> Self {
        Color {
            r: 0x00,
            g: 0x00,
            b: 0x00,
        }
    }

    pub fn from_token(fg: &crate::markup::scan::Token) -> Result<Self, FroggiError> {
        let hex = fg.lexeme().to_owned();

        let rgb = match hex::decode(&hex) {
            Ok(data) => data,
            Err(err) => {
                return Err(FroggiError::markup(
                    MarkupError::IncorrectHexadecimal { hex, err },
                    fg.line(),
                ))
            }
        };

        if rgb.len() != 3 {
            return Err(FroggiError::markup(
                MarkupError::IncorrectColor { color: hex },
                fg.line(),
            ));
        }

        Ok(Color::new(rgb[0], rgb[1], rgb[2]))
    }

    pub fn from_hsv(&self, hue: u16, saturation: u8, value: u8) -> Self {
        let h = hue as f32;
        let s = saturation as f32 / 100.0;
        let v = value as f32 / 100.0;

        let c = v * s;
        let hp = h / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (rp, gp, bp) = {
            if 0.0 <= hp && hp <= 1.0 {
                (c, x, 0.0)
            } else if 1.0 < hp && hp <= 2.0 {
                (x, c, 0.0)
            } else if 2.0 < hp && hp <= 3.0 {
                (0.0, c, x)
            } else if 3.0 < hp && hp <= 4.0 {
                (0.0, x, c)
            } else if 4.0 < hp && hp <= 5.0 {
                (x, 0.0, c)
            } else if 5.0 < hp && hp <= 6.0 {
                (c, 0.0, x)
            } else {
                dbg!(self);
                unreachable!("unknown hue: {}", hp)
            }
        };

        let m = v - c;
        let (r, g, b) = (rp + m, gp + m, bp + m);
        let (r, g, b) = (r * 255.0, g * 255.0, b * 255.0);
        let (r, g, b) = (r.round() as u8, g.round() as u8, b.round() as u8);

        Color { r, g, b }
    }
}
