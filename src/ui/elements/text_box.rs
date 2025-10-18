use macroquad::prelude::*;

use crate::ui::{Bounding, WindowElement};

pub struct TextBox {
    inner_text: String,
    dimensions: TextDimensions,
    font_size: u16,
    font: Font,
    bounding: Bounding,
}
impl TextBox {
    pub fn new(inner_text: &str, font_size: u16, bounding: Bounding) -> Self {
        let font = super::text::main_font();
        let dimensions = measure_text(inner_text, Some(&font), font_size, 1.0);

        Self {
            inner_text: inner_text.to_string(),
            dimensions,
            font_size,
            font,
            bounding,
        }
    }
}
impl WindowElement for TextBox {
    fn get_bounding(&self) -> Bounding {
        self.bounding
    }
    fn update(&mut self, _ctx: &mut crate::ui::WindowContext) {}

    fn draw(&self) {
        draw_text_ex(
            &self.inner_text,
            self.bounding.inner_rect().center().x - self.dimensions.width / 2.0,
            self.bounding.inner_rect().center().y + self.dimensions.height / 2.0,
            TextParams {
                font: Some(&self.font),
                font_size: self.font_size,
                color: BLACK,
                ..Default::default()
            },
        );
    }
}
