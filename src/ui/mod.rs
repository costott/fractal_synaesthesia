use std::sync::{Arc, Mutex};

use macroquad::prelude::*;
use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::{
    project::Project,
    rendering::{
        algorithms::render_algorithms::FractalParams, manager::layer_manager::LayerManager,
    },
    ui::{
        menus::{audio_mapper::AudioMapperMode, fractal_settings::FractalSettingsMode},
        window::*,
    },
};

pub mod fractal;
pub mod menus;
mod window;

pub const NORMAL_TEXT_SIZE: f32 = 15.0;
pub const ACCENT_COLOUR: egui::Color32 = egui::Color32::from_rgb(223, 244, 255);
pub const DARK_ACCENT_COLOUR: egui::Color32 = egui::Color32::from_rgb(50, 75, 100);

enum AppMode {
    FractalSettings,
    AudioMapper,
}

pub struct App {
    mode: AppMode,
    fractal_settings: FractalSettingsMode,
    audio_mapper: AudioMapperMode,

    project: Project,
}
impl App {
    pub fn new() -> Self {
        let project = Project::new();
        Self {
            mode: AppMode::FractalSettings,
            fractal_settings: FractalSettingsMode::new(&project),
            audio_mapper: AudioMapperMode::new(&project),
            project,
        }
    }

    pub fn update(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            match &mut self.mode {
                AppMode::FractalSettings => {
                    let to_swap = self.fractal_settings.update(&mut self.project, egui_ctx);

                    if to_swap {
                        self.mode = AppMode::AudioMapper;
                        self.audio_mapper.changed_fractal_settings(&self.project);
                    }
                }
                AppMode::AudioMapper => {
                    let to_swap = self.audio_mapper.update(&mut self.project, egui_ctx);

                    if to_swap {
                        self.mode = AppMode::FractalSettings;
                    }
                }
            };
        });
    }

    pub fn draw(&self) {
        match &self.mode {
            AppMode::FractalSettings => self.fractal_settings.draw(),
            AppMode::AudioMapper => self.audio_mapper.draw(),
        };
        egui_macroquad::draw();
    }
}

trait AppModeScreen {
    /// Update the screen, and return whether or not the mode should be swapped
    fn update(&mut self, _project: &mut Project, _egui_ctx: &egui::Context) -> bool;
    fn draw(&self);
}

#[derive(Clone)]
pub struct FractalSettings {
    pub params: Arc<Mutex<FractalParams>>,
    pub layers: Arc<Mutex<LayerManager>>,
}
impl Default for FractalSettings {
    fn default() -> Self {
        Self {
            params: Arc::new(Mutex::new(FractalParams::default())),
            layers: Arc::new(Mutex::new(LayerManager::default())),
        }
    }
}
impl Serialize for FractalSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("FractalSettings", 2)?;
        s.serialize_field("params", &*self.params.lock().unwrap())?;
        s.serialize_field("layers", &*self.layers.lock().unwrap())?;
        s.end()
    }
}
impl<'de> Deserialize<'de> for FractalSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FractalSettingsHelper {
            params: FractalParams,
            layers: LayerManager,
        }

        let helper = FractalSettingsHelper::deserialize(deserializer)?;
        Ok(Self {
            params: Arc::new(Mutex::new(helper.params)),
            layers: Arc::new(Mutex::new(helper.layers)),
        })
    }
}

pub trait Dropdown<T>: Clone + PartialEq {
    fn get_variants() -> Vec<T>;
    fn get_text(&self) -> &str;
    fn get_tooltip(&self) -> Option<String> {
        None
    }
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
                let maybe_tooltip = item.get_tooltip();

                let selectable_value = ui.selectable_value(current_val, item, text);

                if let Some(tooltip) = maybe_tooltip {
                    selectable_value.on_hover_text(tooltip);
                }
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
    ctx_rendering: bool,
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
    if !ctx_rendering && !response.has_focus() {
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
