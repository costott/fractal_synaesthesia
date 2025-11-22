use macroquad::prelude::*;

use crate::{
    rendering::video::VideoManager,
    ui::{AppModeScreen, FractalSettings},
};

pub struct AudioMapperMode {
    // top bar: song settings
    // middle section: video preview
    video_manger: Option<VideoManager>,
    // bottom section: audio mapping
}
impl AudioMapperMode {
    pub fn new() -> Self {
        Self { video_manger: None }
    }

    pub fn load_from_fractal_settings(&mut self, settings: FractalSettings) {
        self.video_manger = Some(VideoManager::from_fractal_settings(settings));
    }
}
impl AppModeScreen for AudioMapperMode {
    fn update(&mut self, _egui_ctx: &egui::Context) -> bool {
        false
    }

    fn draw(&self) {
        clear_background(WHITE);
    }
}
