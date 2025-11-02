use crate::ui::{fractal, window::WindowParams};

pub struct LayerManagerSettings {
    params: WindowParams,

    dragging_index: Option<usize>,
    drop_target_index: Option<usize>,
}
impl LayerManagerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            dragging_index: None,
            drop_target_index: None,
        }
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

                        let frame_response = frame
                            .show(ui, |ui| {
                                ui.set_min_width(self.params.width as f32 - 8.0);
                                ui.set_max_width(self.params.width as f32 - 8.0);

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(layer.name.as_str()).font(
                                        egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE),
                                    ));
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("strength").font(
                                            egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE),
                                        ));

                                        let response = ui
                                            .add(egui::Slider::new(&mut layer.strength, 0.0..=1.0));
                                        if response.changed() {
                                            ctx.request_render = true;
                                        }
                                    });

                                    ui.add_space(ui.available_width() - 30.);

                                    let drag_icon = ui.label("⋮⋮");
                                    let drag_icon_response = ui.interact(
                                        drag_icon.interact_rect,
                                        ui.make_persistent_id(format!("drag_icon_{layer_idx}")),
                                        egui::Sense::drag(),
                                    );
                                    if drag_icon_response.drag_started() {
                                        self.dragging_index = Some(layer_idx);
                                        self.drop_target_index = None;
                                    }
                                });
                            })
                            .response;

                        let frame_response_click = ui.interact(
                            frame_response.rect,
                            ui.make_persistent_id(format!("layer_select_{layer_idx}")),
                            egui::Sense::click(),
                        );

                        if frame_response_click.clicked() {
                            *selected_layer = layer_idx;
                            selected_layer_changed = true;
                        }

                        let frame_rect = frame_response.rect;
                        if self.dragging_index.is_some() {
                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                                if frame_rect.contains(pos) {
                                    let insert_idx = if pos.y < frame_rect.center().y {
                                        layer_idx
                                    } else {
                                        layer_idx + 1
                                    };

                                    self.drop_target_index = Some(insert_idx);

                                    ui.painter().hline(
                                        frame_rect.x_range(),
                                        if pos.y < frame_rect.center().y {
                                            frame_rect.top()
                                        } else {
                                            frame_rect.bottom()
                                        },
                                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                    );
                                }
                            }
                        }
                    }
                });

                if ui.input(|i| i.pointer.any_released()) {
                    if let (Some(from), Some(to)) = (self.dragging_index, self.drop_target_index) {
                        let to_move = layer_manager.layers.remove(from);
                        let to_idx = if from < to { to - 1 } else { to };
                        layer_manager.layers.insert(to_idx, to_move);

                        ctx.update_layers = true;
                        ctx.request_render = true;
                    }
                    self.dragging_index = None;
                    self.drop_target_index = None;
                }
            });

        selected_layer_changed
    }
}
