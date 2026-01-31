#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Colour {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgba_u8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        )
    }

    pub fn with_alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub fn interpolate(&self, other: &Colour, t: f32) -> Colour {
        Colour {
            r: lerp(self.r, other.r, t),
            g: lerp(self.g, other.g, t),
            b: lerp(self.b, other.b, t),
            a: lerp(self.a, other.a, t),
        }
    }

    pub fn alpha_blend(&self, fg: &Colour) -> Colour {
        let out_a = fg.a + self.a * (1. - fg.a);

        if out_a == 0. {
            return BLANK;
        }

        Colour {
            r: (fg.r * fg.a + self.r * self.a * (1. - fg.a)) / out_a,
            g: (fg.g * fg.a + self.g * self.a * (1. - fg.a)) / out_a,
            b: (fg.b * fg.a + self.b * self.a * (1. - fg.a)) / out_a,
            a: out_a,
        }
    }
}

pub const BLANK: Colour = Colour {
    r: 0.,
    g: 0.,
    b: 0.,
    a: 0.,
};

pub const BLACK: Colour = Colour {
    r: 0.,
    g: 0.,
    b: 0.,
    a: 1.,
};

pub const WHITE: Colour = Colour {
    r: 1.,
    g: 1.,
    b: 1.,
    a: 1.,
};

/// Linear interpolation between `a` to `b` with parameter `t`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1f32 - t) * a + t * b
}

impl From<macroquad::color::Color> for Colour {
    fn from(value: macroquad::color::Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}
impl From<Colour> for macroquad::color::Color {
    fn from(value: Colour) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}
impl From<egui::Color32> for Colour {
    fn from(value: egui::Color32) -> Self {
        Self {
            r: value.r() as f32 / 255.,
            g: value.g() as f32 / 255.,
            b: value.b() as f32 / 255.,
            a: value.a() as f32 / 255.,
        }
    }
}
impl From<Colour> for egui::Color32 {
    fn from(value: Colour) -> Self {
        Self::from_rgba_premultiplied(
            (value.r * 255.) as u8,
            (value.g * 255.) as u8,
            (value.b * 255.) as u8,
            (value.a * 255.) as u8,
        )
    }
}
impl From<egui::Rgba> for Colour {
    fn from(value: egui::Rgba) -> Self {
        fn linear_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }

        Colour::new(
            linear_to_srgb(value.r()),
            linear_to_srgb(value.g()),
            linear_to_srgb(value.b()),
            value.a(),
        )
    }
}
impl From<Colour> for egui::Rgba {
    fn from(value: Colour) -> Self {
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        egui::Rgba::from_rgba_premultiplied(
            srgb_to_linear(value.r),
            srgb_to_linear(value.g),
            srgb_to_linear(value.b),
            value.a,
        )
    }
}
