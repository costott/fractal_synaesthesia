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
pub mod audio_mapping_window;
use audio_mapping_window::AudioMappingWindow;
mod audio_player;
mod waveform;

pub struct AudioMapperMode {
    context: AudioMapperContext,

    // -------------------------------------
    // top bar: song settings
    song_settings: SongSettings,
    // middle section: video preview
    video_manger: VideoManager,
    video_manager_window: VideoPreviewWindow,
    // bottom section: audio mapping
    audio_mapping_window: AudioMappingWindow,
    // -------------------------------------
    /// Stored for use in restoring settings back into fractal settings mode
    end_fractal_settings: FractalSettings,
}
impl AudioMapperMode {
    pub fn new(settings: FractalSettings) -> Self {
        let context = AudioMapperContext::load_from_fractal_settings(&settings);

        let preview_dims = context
            .video_dimensions
            .new_from_this_aspect_with_height(345);

        let video_preview_params = WindowParams {
            width: (screen_width() * 0.4) as u16,
            height: preview_dims.height + 20 + 30,
            x: (screen_width() * 0.3) as u16,
            y: 61,
        };

        Self {
            song_settings: SongSettings::new(WindowParams {
                width: screen_width() as u16,
                height: 60,
                x: 0,
                y: 0,
            }),
            // TODO: change framerate and hop size
            video_manger: VideoManager::new(&context, &settings.params, 30., 1.),
            video_manager_window: VideoPreviewWindow::new(
                video_preview_params.clone(),
                &context,
                preview_dims,
                &settings.params,
            ),
            audio_mapping_window: AudioMappingWindow::new(WindowParams {
                width: screen_width() as u16,
                height: screen_height() as u16
                    - (video_preview_params.y + video_preview_params.height),
                x: 0,
                y: video_preview_params.y + video_preview_params.height,
            }),
            context,
            end_fractal_settings: settings,
        }
    }
}
impl AppModeScreen for AudioMapperMode {
    fn update(&mut self, egui_ctx: &egui::Context) -> bool {
        egui_ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(egui::Color32::WHITE);
            style.visuals.extreme_bg_color = egui::Color32::DARK_GRAY;
            style.visuals.widgets.inactive.bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.hovered.bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.active.bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.active.weak_bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.open.bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.widgets.open.weak_bg_fill = egui::Color32::DARK_GRAY;
            style.visuals.window_fill = egui::Color32::DARK_GRAY;
            style.visuals.selection.bg_fill = crate::ui::DARK_ACCENT_COLOUR;
        });

        if self.song_settings.update(egui_ctx, &mut self.context) {
            self.video_manger.updated_context(&self.context);
            self.video_manager_window.updated_context(&self.context);
            self.video_manger.updated_audio_mapper(&self.context);
        }

        self.video_manager_window
            .update(egui_ctx, &mut self.context, &mut self.video_manger);

        if self
            .audio_mapping_window
            .update(egui_ctx, &mut self.context)
        {
            self.video_manger.updated_audio_mapper(&self.context);
        }

        false
    }

    fn draw(&self) {
        clear_background(Color {
            r: 0.09,
            g: 0.09,
            b: 0.09,
            a: 1.,
        });

        self.video_manager_window.draw();
    }
}

pub struct AudioMapperContext {
    pub audio_mapper: AudioMapper,
    pub layer_manager: Arc<Mutex<LayerManager>>,
    pub video_dimensions: CanvasDimensions,
    pub song_manager: Option<SongManager>,
    pub song_path: Option<String>,
}
impl AudioMapperContext {
    pub fn load_from_fractal_settings(settings: &FractalSettings) -> Self {
        Self {
            audio_mapper: AudioMapper::empty(),
            layer_manager: Arc::new(Mutex::new(settings.layers.clone())),
            video_dimensions: CanvasDimensions {
                width: 1920,
                height: 1080,
            },
            song_manager: None,
            song_path: None,
        }
    }
}

pub trait AudioMapperWindow {
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut AudioMapperContext);
}
