use std::sync::{Arc, Mutex};

use macroquad::prelude::*;

use crate::{
    rendering::{
        manager::layer_manager::LayerManager,
        video::{VideoManager, audio_mapper::AudioMapper, song_manager::SongManager},
    },
    ui::{
        AppModeScreen, FractalSettings, fractal::fractal_canvas::CanvasDimensions,
        window::WindowParams,
    },
};

mod video_preview_window;
use video_preview_window::VideoPreviewWindow;
mod song_settings;
use song_settings::SongSettings;
mod audio_player;

pub struct AudioMapperMode {
    context: AudioMapperContext,

    // -------------------------------------
    // top bar: song settings
    song_settings: SongSettings,
    // middle section: video preview
    video_manger: Option<VideoManager>,
    video_manager_window: VideoPreviewWindow,
    // bottom section: audio mapping
    audio_mapper: AudioMapper,
    // -------------------------------------
    /// Stored for use in restoring settings back into fractal settings mode
    end_fractal_settings: FractalSettings,
}
impl AudioMapperMode {
    pub fn new(settings: FractalSettings) -> Self {
        let context = AudioMapperContext::load_from_fractal_settings(&settings);
        Self {
            song_settings: SongSettings::new(WindowParams {
                width: screen_width() as u16,
                height: 60,
                x: 0,
                y: 0,
            }),
            video_manger: None,
            video_manager_window: VideoPreviewWindow {},
            audio_mapper: AudioMapper::empty(),
            context,
            end_fractal_settings: settings,
        }
    }
}
impl AppModeScreen for AudioMapperMode {
    fn update(&mut self, egui_ctx: &egui::Context) -> bool {
        self.song_settings.update(egui_ctx, &mut self.context);

        false
    }

    fn draw(&self) {
        clear_background(Color {
            r: 0.09,
            g: 0.09,
            b: 0.09,
            a: 1.,
        });
    }
}

#[derive(Clone)]
pub struct AudioMapperContext {
    pub layer_manager: Arc<Mutex<LayerManager>>,
    pub video_dimensions: CanvasDimensions,
    pub song_manager: Option<SongManager>,
}
impl AudioMapperContext {
    pub fn load_from_fractal_settings(settings: &FractalSettings) -> Self {
        Self {
            layer_manager: Arc::new(Mutex::new(settings.layers.clone())),
            video_dimensions: CanvasDimensions {
                width: 800,
                height: 600,
            },
            song_manager: None,
        }
    }
}

pub trait AudioMapperWindow {
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut AudioMapperContext);
}
