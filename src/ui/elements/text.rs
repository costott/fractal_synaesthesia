use macroquad::prelude::*;

pub fn main_font() -> Font {
    load_ttf_font_from_bytes(include_bytes!("../Montserrat-SemiBold.ttf")).unwrap()
}
