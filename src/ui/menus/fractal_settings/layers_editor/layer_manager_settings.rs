use crate::ui::window::WindowParams;

pub struct LayerManagerSettings {
    params: WindowParams,
}
impl LayerManagerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }

    /// Returns whether the selected layer changed
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut crate::ui::window::WindowContext,
        selected_layer: &mut usize,
    ) -> bool {
        let mut selected_layer_changed = false;

        self.params
            .sized_area("layer_manager_settings", egui_ctx, |ui| {
                ui.painter().line_segment(
                    [
                        egui::Pos2::new(ui.max_rect().left(), ui.max_rect().top()),
                        egui::Pos2::new(ui.max_rect().right(), ui.max_rect().top()),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::BLACK),
                );

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

                ui.add_space(5.0);

                let layer_manager = &mut ctx.layer_manager.lock().unwrap();

                // TODO: cannot add during rendering
                if ui.button("+").clicked() && !ctx.update_layers {
                    layer_manager.add_layer();
                    ctx.update_layers = true;
                    ctx.request_render = true;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (layer_idx, layer) in layer_manager.layers.iter_mut().enumerate() {
                        let frame = egui::Frame::new()
                            .fill(if *selected_layer == layer_idx {
                                crate::ui::ACCENT_COLOUR
                            } else {
                                egui::Color32::WHITE
                            })
                            .inner_margin(egui::Margin::symmetric(3, 5))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY));

                        let frame_rect = ui.available_rect_before_wrap();
                        frame.show(ui, |ui| {
                            ui.set_min_width(self.params.width as f32 - 8.0);
                            ui.set_max_width(self.params.width as f32 - 8.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(layer.name.as_str()).font(
                                        egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE),
                                    ),
                                );
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("strength").font(
                                        egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE),
                                    ));

                                    let response =
                                        ui.add(egui::Slider::new(&mut layer.strength, 0.0..=1.0));
                                    if response.changed() {
                                        ctx.request_render = true;
                                    }
                                });
                            });
                        });

                        let frame_response_click = ui.interact(
                            frame_rect,
                            ui.make_persistent_id(("layer_reorder", layer_idx)),
                            egui::Sense::click(),
                        );
                        if frame_response_click.clicked() {
                            *selected_layer = layer_idx;
                            selected_layer_changed = true;
                        }
                    }
                });
            });

        selected_layer_changed
    }
}
