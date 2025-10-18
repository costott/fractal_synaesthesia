use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::rendering::algorithms::render_algorithms::FractalParams;

pub trait Window {
    fn is_open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut WindowContext);
}

#[derive(Clone, Copy)]
pub struct WindowParams {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
}
impl WindowParams {
    pub fn get_bounding_rect(&self) -> Rect {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

pub struct WindowContext {
    pub fractal_params: Arc<Mutex<FractalParams>>,
    pub request_render: bool,
}
