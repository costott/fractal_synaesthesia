use crate::ui::{menus::fractal_settings::FractalSettingsWindow, window::WindowParams};

mod layer_settings;
use layer_settings::LayerSettings;
mod layer_manager_settings;
use layer_manager_settings::LayerManagerSettings;
mod palette_editor;

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
impl FractalSettingsWindow for LayersEditor {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) {
        self.params.sized_area("layers_editor", egui_ctx, |_| {
            let is_changed =
                self.layer_settings
                    .update(egui_ctx, project, ctx, self.selected_layer);

            if is_changed {
                self.layer_manager_settings
                    .layer_changed(self.selected_layer);
                ctx.update_layers = true;
                ctx.request_render = true;
            }

            let selected_layer_changed = self.layer_manager_settings.update(
                egui_ctx,
                project,
                ctx,
                &mut self.selected_layer,
            );

            if selected_layer_changed {
                self.layer_settings.selected_layer_changed();
            }
        });
    }
}
