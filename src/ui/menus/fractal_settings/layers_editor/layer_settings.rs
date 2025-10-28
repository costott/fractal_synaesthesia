use crate::{
    rendering::manager::layer::LayerAlgorithmKind,
    ui::{Dropdown, window::WindowParams},
};

pub struct LayerSettings {
    params: WindowParams,
}
impl LayerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }

    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut crate::ui::window::WindowContext,
        selected_layer: usize,
    ) {
        self.params.sized_area("layer_settings", egui_ctx, |ui| {
            let layer = &mut ctx.layer_manager.lock().unwrap().layers[selected_layer];

            ui.label(egui::RichText::new(layer.name.clone()).heading());

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Formula")
                        .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                );

                let before = layer.algorithm.clone();

                egui::ComboBox::from_id_salt("formula")
                    .selected_text(layer.algorithm.get_text())
                    .show_ui(ui, |ui| {
                        for item in LayerAlgorithmKind::get_variants() {
                            let text = item.get_text().to_string();
                            ui.selectable_value(&mut layer.algorithm, item, text);
                        }
                    });

                if layer.algorithm != before {
                    ctx.update_layers = true;
                    ctx.request_render = true;
                }
            });
        });
    }
}
