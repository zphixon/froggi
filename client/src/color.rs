#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    hue: u16,
    saturation: u8,
    value: u8,
}

impl Color {
    pub fn new(hue: u16, saturation: u8, value: u8) -> Self {
        assert!(hue <= 360);
        assert!(saturation <= 100);
        assert!(value <= 100);

        Self {
            hue,
            saturation,
            value,
        }
    }

    pub fn white() -> Self {
        Self {
            hue: 0,
            saturation: 0,
            value: 100,
        }
    }

    pub fn black() -> Self {
        Self {
            hue: 0,
            saturation: 0,
            value: 0,
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        use float_eq::float_eq;

        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let x_max = r.max(g.max(b));
        let x_min = r.min(g.min(b));

        let c = x_max - x_min;

        let h = if float_eq!(c, 0.0, abs <= 0.000_1) {
            0.0
        } else if float_eq!(x_max, r, ulps <= 4) {
            60.0 * ((g - b) / c)
        } else if float_eq!(x_max, g, ulps <= 4) {
            60.0 * (2.0 + ((b - r) / c))
        } else if float_eq!(x_max, b, ulps <= 4) {
            60.0 * (4.0 + ((r - g) / c))
        } else {
            unreachable!()
        };

        let s = if float_eq!(x_max, 0.0, abs <= 0.000_1) {
            0.0
        } else {
            c / x_max
        };

        let hue = h.round() as u16;
        let saturation = (s * 100.0).round() as u8;
        let value = (x_max * 100.0).round() as u8;
        Self {
            hue,
            saturation,
            value,
        }
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let h = self.hue as f32;
        let s = self.saturation as f32 / 100.0;
        let v = self.value as f32 / 100.0;

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
        (r.round() as u8, g.round() as u8, b.round() as u8)
    }

    pub fn hue(&self) -> &u16 {
        &self.hue
    }

    pub fn saturation(&self) -> &u8 {
        &self.saturation
    }

    pub fn value(&self) -> &u8 {
        &self.value
    }
}
