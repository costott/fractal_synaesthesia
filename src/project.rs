use serde::{Deserialize, Serialize};

use crate::{
    rendering::video::audio_mapper::AudioMapper,
    ui::{FractalSettings, fractal::fractal_canvas::CanvasDimensions},
};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub fractal_settings: FractalSettings,
    pub song_path: Option<String>,
    pub video_dimensions: CanvasDimensions,
    fps: usize,
    pub audio_mapper: AudioMapper,
}
impl Project {
    pub fn new() -> Self {
        Self {
            fractal_settings: FractalSettings::default(),
            song_path: None,
            video_dimensions: CanvasDimensions {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio_mapper: AudioMapper::empty(),
        }
    }

    pub fn fps(&self) -> usize {
        self.fps
    }

    pub fn set_fps(&mut self, fps: usize) {
        self.fps = fps;
    }
}
