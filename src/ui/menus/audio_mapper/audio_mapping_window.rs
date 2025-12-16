use crate::{rendering::video::audio_mapper::LayerAction, ui::window::WindowParams};

pub struct AudioMappingWindow {
    params: WindowParams,
}
impl AudioMappingWindow {
    const MAPPING_PARAM_HEIGHT: f32 = 20.0;

    pub fn new(params: WindowParams) -> Self {
        AudioMappingWindow { params }
    }

    /// Renders a feature window editor for the given window
    ///
    /// Returns true if any changes were made to the zoom timeline
    fn feature_window_editor(
        ui: &mut egui::Ui,
        heading: &str,
        window: &mut crate::rendering::video::audio_mapper::ContinuousSampleWindow,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Window duration:").on_hover_text(
                "The duration of audio samples around the timestamp to consider for this feature",
            );
            let duration_response = ui.add(
                egui::DragValue::new(&mut window.duration)
                    .speed(0.01)
                    .range(0.0..=f32::MAX),
            );
            if duration_response.changed() {
                changed = true;
            }

            ui.add_space(10.0);

            ui.label("Window type:");
            changed |= crate::ui::show_dropdown(
                ui,
                &mut window.window_type,
                format!("{}_window_type_dropdown", heading.to_lowercase()),
            );
        });

        changed
    }

    /// Renders a feature mapper section for the given heading and layers
    ///
    /// Returns true if any changes were made to the zoom timeline
    fn feature_mapper(
        ui: &mut egui::Ui,
        heading: &str,
        layers: &[crate::rendering::manager::layer::Layer],
        layer_actions: &mut std::collections::HashMap<usize, Vec<LayerAction>>,
    ) -> bool {
        let mut changed = false;

        egui::ScrollArea::vertical()
            .id_salt(format!("audio_mapper_scroll_{}", heading))
            .show(ui, |ui| {
                for (i, layer) in layers.iter().enumerate() {
                    let layer_frame = egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(3, 5))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY));

                    // call the mapper once and reuse the result
                    let n_mappings = layer_actions.get(&i).as_ref().map(|a| a.len()).unwrap_or(0);
                    let height = (n_mappings + 1) as f32 * Self::MAPPING_PARAM_HEIGHT;

                    layer_frame.show(ui, |ui| {
                        ui.set_min_height(height);
                        ui.set_max_height(height);

                        ui.horizontal_centered(|ui| {
                            ui.label(&layer.name);
                            ui.vertical(|ui| {
                                let action_frame =
                                    egui::Frame::new().inner_margin(egui::Margin::symmetric(2, 2));

                                let mut remove_action_index = None;
                                if let Some(actions) = layer_actions.get_mut(&i) {
                                    let n_actions = actions.len();
                                    for (i, action) in actions.iter_mut().enumerate() {
                                        action_frame.clone().show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.set_min_width(ui.available_width() - 7.0);
                                                ui.set_max_width(ui.available_width() - 7.0);
                                                ui.set_min_height(Self::MAPPING_PARAM_HEIGHT);

                                                crate::ui::show_dropdown(
                                                    ui,
                                                    action,
                                                    format!(
                                                        "{}_action_dropdown_{i}",
                                                        heading.to_lowercase()
                                                    ),
                                                );

                                                let value_changed = action.edit_inner_value(ui);
                                                if value_changed && action.changes_zoom_timeline() {
                                                    changed = true;
                                                }

                                                ui.add_space(
                                                    ui.available_width()
                                                        - crate::ui::NORMAL_TEXT_SIZE,
                                                );

                                                if ui.button("x").clicked() {
                                                    remove_action_index = Some(i);
                                                }
                                            });
                                        });
                                    }
                                }

                                if let Some(index) = remove_action_index {
                                    if let Some(actions) = layer_actions.get_mut(&i) {
                                        let removed_action = actions.remove(index);
                                        if removed_action.changes_zoom_timeline() {
                                            changed = true;
                                        }
                                    }
                                }

                                let plus_response = {
                                    // allocate an exact area we can make clickable
                                    let size = egui::Vec2::new(
                                        ui.available_width() - 7.0,
                                        Self::MAPPING_PARAM_HEIGHT,
                                    );
                                    let (rect, response) =
                                        ui.allocate_exact_size(size, egui::Sense::click());

                                    let normal = ui.style().visuals.widgets.inactive.bg_fill;
                                    let hover = ui.style().visuals.widgets.active.bg_fill;
                                    let bg = if response.hovered() { hover } else { normal };
                                    let stroke_color = if response.hovered() {
                                        ui.style().visuals.widgets.active.fg_stroke.color
                                    } else {
                                        egui::Color32::DARK_GRAY
                                    };

                                    // draw background and border
                                    ui.painter().rect_filled(rect, 2.0, bg);
                                    ui.painter().rect_stroke(
                                        rect,
                                        2.0,
                                        egui::Stroke::new(1.0, stroke_color),
                                        egui::StrokeKind::Middle,
                                    );

                                    // draw the plus sign centered
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "+",
                                        egui::FontId::proportional(16.0),
                                        ui.style().visuals.text_color(),
                                    );

                                    response
                                };

                                if plus_response.clicked() {
                                    if let Some(actions) = layer_actions.get_mut(&i) {
                                        actions.push(LayerAction::default());
                                        changed = true;
                                    } else {
                                        layer_actions.insert(i, vec![LayerAction::default()]);
                                    }
                                }
                            });
                        })
                    });
                }
            });

        changed
    }

    /// Updates the audio mapping window UI
    ///
    /// Returns true if any changes were made to the zoom timeline
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut super::AudioMapperContext,
    ) -> bool {
        let mut changed = false;

        self.params.sized_area("audio mappings", egui_ctx, |ui| {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().left(), ui.max_rect().top()),
                    egui::Pos2::new(ui.max_rect().right(), ui.max_rect().top()),
                ],
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            let layer_manager = ctx.layer_manager.lock().unwrap();
            let layers = layer_manager.layers.clone();
            drop(layer_manager);

            ui.columns(4, |columns| {
                // add line at the right of the first column
                for col in 0..=2 {
                    columns[col].painter().line_segment(
                        [
                            egui::Pos2::new(
                                columns[col].max_rect().right(),
                                columns[col].max_rect().top(),
                            ),
                            egui::Pos2::new(
                                columns[col].max_rect().right(),
                                columns[col].max_rect().bottom(),
                            ),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                    );
                }

                columns[0]
                    .heading("On Beat")
                    .on_hover_text("Actions to perform on each layer when a beat is detected");
                columns[0].horizontal(|ui| {
                    ui.label("Attack duration:").on_hover_text(
                        "The duration before beats over which the 'on beat' effect will ramp up",
                    );
                    ui.add(
                        egui::DragValue::new(ctx.audio_mapper.on_beat_attack_duration_mut())
                            .speed(0.01)
                            .range(0.0..=1.0),
                    );

                    ui.add_space(10.0);
                    ui.label("Decay duration:").on_hover_text(
                        "The duration after beats over which the 'on beat' effect will ramp down",
                    );
                    ui.add(
                        egui::DragValue::new(ctx.audio_mapper.on_beat_decay_duration_mut())
                            .speed(0.01)
                            .range(0.0..=1.0),
                    );
                });
                changed |= Self::feature_mapper(
                    &mut columns[0],
                    "On Beat",
                    &layers,
                    ctx.audio_mapper.on_beat_layer_actions_mut(),
                );

                columns[1]
                    .heading("Tempo")
                    .on_hover_text("Actions to perform on each layer based on the tempo");
                changed |= Self::feature_window_editor(
                    &mut columns[1],
                    "Tempo",
                    ctx.audio_mapper.tempo_window_mut(),
                );
                changed |= Self::feature_mapper(
                    &mut columns[1],
                    "Tempo",
                    &layers,
                    ctx.audio_mapper.tempo_layer_actions_mut(),
                );

                columns[2]
                    .heading("Volume")
                    .on_hover_text("Actions to perform on each layer based on the volume");
                changed |= Self::feature_window_editor(
                    &mut columns[2],
                    "Volume",
                    ctx.audio_mapper.volume_window_mut(),
                );
                changed |= Self::feature_mapper(
                    &mut columns[2],
                    "Volume",
                    &layers,
                    ctx.audio_mapper.volume_layer_actions_mut(),
                );

                columns[3]
                    .heading("Pitch")
                    .on_hover_text("Actions to perform on each layer based on the pitch");
                changed |= Self::feature_window_editor(
                    &mut columns[3],
                    "Pitch",
                    ctx.audio_mapper.pitch_window_mut(),
                );
                changed |= Self::feature_mapper(
                    &mut columns[3],
                    "Pitch",
                    &layers,
                    ctx.audio_mapper.pitch_layer_actions_mut(),
                );
            });
        });

        changed
    }
}
pub trait MappingAction {
    /// Edit the inner value of this action in the given UI
    ///
    /// Returns true if any changes were made
    fn edit_inner_value(&mut self, ui: &mut egui::Ui) -> bool;
}
