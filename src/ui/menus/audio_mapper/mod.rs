use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

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
mod video_settings;
use video_settings::VideoSettings;
mod export_menu;
use export_menu::ExportMenu;
mod exporting_modal;
use exporting_modal::ExportingModal;
mod waveform;

pub const BACKGROUND_PRIMARY_COLOUR: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);
pub const BACKGROUND_SECONDARY_COLOUR: egui::Color32 = egui::Color32::from_rgb(50, 50, 50);

pub struct AudioMapperMode {
    context: AudioMapperContext,

    // -------------------------------------
    // top bar: song settings
    song_settings: SongSettings,
    // middle section: video preview + settings
    video_preview_window: VideoPreviewWindow,

    export_menu: ExportMenu,
    video_settings: VideoSettings,
    exporting_modal: Option<ExportingModal>,
    // bottom section: audio mapping
    audio_mapping_window: AudioMappingWindow,
    // -------------------------------------
    /// Stored for use in restoring settings back into fractal settings mode
    end_fractal_settings: FractalSettings,
}
impl AudioMapperMode {
    pub fn new(settings: FractalSettings) -> Self {
        let context = AudioMapperContext::load_from_fractal_settings(&settings);

        let video_preview_params = WindowParams {
            width: (screen_width() * 0.4) as u16,
            height: 345 + 20 + 30,
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
            video_preview_window: VideoPreviewWindow::new(
                video_preview_params.clone(),
                &context,
                context.video_dimensions,
                345,
                &settings.params,
            ),
            export_menu: ExportMenu::new(WindowParams {
                width: (screen_width() * 0.3) as u16,
                height: video_preview_params.height,
                x: 0,
                y: video_preview_params.y,
            }),
            video_settings: VideoSettings::new(WindowParams {
                width: (screen_width() * 0.3) as u16,
                height: video_preview_params.height,
                x: video_preview_params.x + video_preview_params.width,
                y: video_preview_params.y,
            }),
            exporting_modal: None,
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

        if self.context.exporting {
            if let Some(exporting_modal) = &mut self.exporting_modal {
                exporting_modal.update(egui_ctx, &mut self.context);
            } else {
                self.exporting_modal = Some(ExportingModal::new(
                    &self.end_fractal_settings.params,
                    &self.context,
                ));
            }
        }

        if self.song_settings.update(egui_ctx, &mut self.context) {
            if let Some(sm) = self.context.song_manager.as_ref() {
                self.context
                    .video_manager
                    .updated_duration(sm.song.duration());
            }
            self.video_preview_window.updated_context(&self.context);
            self.context.video_manager.updated_audio_mapper(
                self.context.song_manager.as_ref(),
                self.context.layer_manager.clone(),
                &self.context.audio_mapper,
            );
        }

        if self.video_settings.update(egui_ctx, &mut self.context) {
            self.video_preview_window
                .change_dimensions(self.context.video_dimensions);
        }

        self.video_preview_window
            .update(egui_ctx, &mut self.context);

        if self
            .audio_mapping_window
            .update(egui_ctx, &mut self.context)
        {
            self.context.video_manager.updated_audio_mapper(
                self.context.song_manager.as_ref(),
                self.context.layer_manager.clone(),
                &self.context.audio_mapper,
            );
        }

        self.export_menu.update(egui_ctx, &mut self.context);

        false
    }

    fn draw(&self) {
        clear_background(Color {
            r: BACKGROUND_PRIMARY_COLOUR.r() as f32 / 255.,
            g: BACKGROUND_PRIMARY_COLOUR.g() as f32 / 255.,
            b: BACKGROUND_PRIMARY_COLOUR.b() as f32 / 255.,
            a: 1.,
        });

        self.video_preview_window.draw();
    }
}

pub struct AudioMapperContext {
    pub audio_mapper: AudioMapper,
    pub video_manager: VideoManager,
    pub layer_manager: Arc<Mutex<LayerManager>>,
    pub video_dimensions: CanvasDimensions,
    pub song_manager: Option<SongManager>,
    pub song_path: Option<String>,
    pub fps: usize,

    pub exporting: bool,
    pub destination_file_path: Option<PathBuf>,
    pub intermediate_pngs: bool,
}
impl AudioMapperContext {
    pub fn load_from_fractal_settings(settings: &FractalSettings) -> Self {
        Self {
            audio_mapper: AudioMapper::empty(),
            video_manager: VideoManager::new(&settings.params, 60., 30., 1.),
            layer_manager: Arc::new(Mutex::new(settings.layers.clone())),
            video_dimensions: CanvasDimensions {
                width: 1920,
                height: 1080,
            },
            song_manager: None,
            song_path: None,
            fps: 30,
            exporting: false,
            // set default destination to downloads folder
            destination_file_path: dirs::video_dir().map(|mut path| {
                path.push("output.mp4");
                path
            }),
            intermediate_pngs: true,
        }
    }

    pub fn change_fps(&mut self, fps: usize) {
        self.fps = fps;
        self.video_manager.update_fps(
            fps as f32,
            self.song_manager.as_ref(),
            self.layer_manager.clone(),
            &self.audio_mapper,
        );
    }
}

pub trait AudioMapperWindow {
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut AudioMapperContext);
}
