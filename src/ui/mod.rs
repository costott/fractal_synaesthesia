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
                style.visuals.widgets.hovered.bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.active.bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.active.weak_bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.open.bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.open.weak_bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.window_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.selection.bg_fill = crate::ui::ACCENT_COLOUR;
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

pub trait Dropdown<T>: Clone + PartialEq {
    fn get_variants() -> Vec<T>;
    fn get_text(&self) -> &str;
}

/// Displays a `Combobox` and returns whether the current value was changed
fn show_dropdown<T>(ui: &mut egui::Ui, current_val: &mut T, id_salt: impl std::hash::Hash) -> bool
where
    T: Dropdown<T>,
{
    let before = current_val.clone();
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(current_val.get_text())
        .show_ui(ui, |ui| {
            for item in T::get_variants() {
                let text = item.get_text().to_string();
                ui.selectable_value(current_val, item, text);
            }
        });
    *current_val != before
}

/// Displays a labeled single-line text input for editing a parameter, and
/// synchronizes its value with external state via provided getter and setter closures.
///
/// This helper abstracts the common egui boilerplate for numeric or string parameters
/// that are represented as editable text. It handles focus changes, updates the
/// underlying value when the field loses focus, and refreshes the displayed text
/// when the field is not focused.
///
/// # Arguments
///
/// * `ui`: The egui [`egui::Ui`] instance to draw into.
/// * `label`: The text label to display beside the input box.
/// * `local_value` A mutable reference to the locally cached string representation
///   of the parameter. This value is edited directly by the user.
/// * `get_value`: A closure returning the current string representation of the
///   external parameter, used to refresh `local_value` when the text field is not focused.
/// * `set_value`: — closure that takes the newly entered string and applies it
///   to the external parameter when the text field loses focus.
///
/// # Returns
///
/// Returns `true` if the external parameter was modified and a re-render
/// should be triggered, otherwise returns `false`.
fn text_param<FGet, FSet>(
    ui: &mut egui::Ui,
    label: &str,
    local_value: &mut String,
    get_value: FGet,
    mut set_value: FSet,
) -> bool
where
    FGet: Fn() -> String,
    FSet: FnMut(String),
{
    ui.label(
        egui::RichText::new(label).font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
    );

    let response = ui.text_edit_singleline(local_value);
    let mut request_render = false;

    // Set render value when sumbitted
    if response.lost_focus() {
        set_value(local_value.clone());
        request_render = true;
    }

    // Update UI value when not being changed by UI
    if !response.has_focus() {
        *local_value = get_value();
    }

    ui.end_row();
    request_render
}

fn get_texture_handle_from_texture2d(
    egui_ctx: &egui::Context,
    texture2d: Texture2D,
    name: impl Into<String>,
) -> egui::TextureHandle {
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [texture2d.width() as usize, texture2d.height() as usize],
        &texture2d.get_texture_data().bytes,
    );
    egui_ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
}

fn draw_image_from_handle(
    ui: &mut egui::Ui,
    handle: &egui::TextureHandle,
) -> (egui::Rect, egui::Response) {
    let t = egui::load::SizedTexture::from_handle(&handle);
    let image = egui::Image::from_texture(t);
    let (rect, response) =
        ui.allocate_exact_size(image.size().unwrap(), egui::Sense::click_and_drag());
    image.paint_at(ui, rect);

    (rect, response)
}
