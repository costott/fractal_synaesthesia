use crate::ui::window::{Window, WindowParams};

pub struct LayerManagerSettings {
    params: WindowParams,
    is_open: bool,
}
impl LayerManagerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            is_open: true,
        }
    }

    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        _ctx: &mut crate::ui::window::WindowContext,
        selected_layer: &mut usize,
    ) {
        self.params
            .sized_area("layer_manager_settings", egui_ctx, |ui| {
                egui::Frame::new()
                    .fill(crate::ui::ACCENT_COLOUR)
                    .inner_margin(egui::Margin::symmetric(0, 6))
                    .show(ui, |ui| {
                        ui.set_min_width(self.params.width as f32);
                        ui.set_max_width(self.params.width as f32);

                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Manager").heading());
                        });
                    });
            });
    }
}
