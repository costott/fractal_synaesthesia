use crate::ui::window::{Window, WindowParams};

mod layer_settings;
use layer_settings::LayerSettings;
mod layer_manager_settings;
use layer_manager_settings::LayerManagerSettings;

pub struct LayersEditor {
    params: WindowParams,
    is_open: bool,

    layer_settings: LayerSettings,
    layer_manager_settings: LayerManagerSettings,

    /// Index of the selected layer in the layer manager
    selected_layer: usize,
}
impl LayersEditor {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            is_open: true,
            layer_settings: LayerSettings::new(WindowParams {
                width: params.width - 5,
                height: params.height / 2,
                x: params.x + 5,
                y: params.y,
            }),
            layer_manager_settings: LayerManagerSettings::new(WindowParams {
                width: params.width,
                height: params.height / 2,
                x: params.x,
                y: params.y + params.height / 2,
            }),
            selected_layer: 0,
        }
    }
}
impl Window for LayersEditor {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }

    fn update(&mut self, egui_ctx: &egui::Context, ctx: &mut crate::ui::window::WindowContext) {
        self.params.sized_area("layers_editor", egui_ctx, |ui| {
            self.layer_settings
                .update(egui_ctx, ctx, self.selected_layer);
            self.layer_manager_settings
                .update(egui_ctx, ctx, &mut self.selected_layer);
        });
    }
}
