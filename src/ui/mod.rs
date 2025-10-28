use macroquad::prelude::*;

use crate::ui::{menus::fractal_settings::FractalSettingsMode, window::*};

pub mod fractal;
mod menus;
mod window;

pub const NORMAL_TEXT_SIZE: f32 = 15.0;
pub const ACCENT_COLOUR: egui::Color32 = egui::Color32::from_rgb(223, 244, 255);

enum AppMode {
    FractalSettings,
    AudioMapper,
}

pub struct App {
    mode: AppMode,

    fractal_settings: FractalSettingsMode,
    audio_mapper: AudioMapperMode,
}
impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::FractalSettings,
            fractal_settings: FractalSettingsMode::new(),
            audio_mapper: AudioMapperMode::new(),
        }
    }

    pub fn update(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.style_mut(|style| {
                style.visuals.override_text_color = Some(egui::Color32::DARK_GRAY);
                style.visuals.extreme_bg_color = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.inactive.bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            });

            match self.mode {
                AppMode::FractalSettings => self.fractal_settings.update(egui_ctx),
                AppMode::AudioMapper => self.audio_mapper.update(egui_ctx),
            };
        });
    }

    pub fn draw(&self) {
        match self.mode {
            AppMode::FractalSettings => self.fractal_settings.draw(),
            AppMode::AudioMapper => self.audio_mapper.draw(),
        };
        egui_macroquad::draw();
    }
}

trait AppModeScreen {
    fn update(&mut self, _egui_ctx: &egui::Context) {}
    fn draw(&self) {}
}

pub struct AudioMapperMode {}
impl AudioMapperMode {
    pub fn new() -> Self {
        Self {}
    }
}
impl AppModeScreen for AudioMapperMode {}

pub trait Dropdown<T> {
    fn get_variants() -> Vec<T>;
    fn get_text(&self) -> &str;
}
