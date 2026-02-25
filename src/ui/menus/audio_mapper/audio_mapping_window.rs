use crate::{
    rendering::video::audio_mapper::{IntensityEmphasis, MapperType, OnBeatMapper},
    ui::window::WindowParams,
};

pub struct AudioMappingWindow {
    params: WindowParams,
}
impl AudioMappingWindow {
    const MAPPING_PARAM_HEIGHT: f32 = 18.0;
    const MIN_MAPPING_HEIGHT: f32 = 60.0;
    const MAPPING_WIDTH: f32 = 350.0;
    const MAPPING_DELETE_WIDTH: f32 = 20.0;
    const LAYER_INNER_X_MARGIN: f32 = 10.0;
    const LAYER_OUTER_X_MARGIN: f32 = 5.0;

    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }

    /// Updates the audio mapping window UI
    ///
    /// Returns true if any changes were made to a timeline
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        _ctx: &mut super::AudioMapperContext,
    ) -> bool {
        let mut changed = false;

        self.params.sized_area("audio mappings", egui_ctx, |ui| {
            // Line at top of window
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().left(), ui.max_rect().top()),
                    egui::Pos2::new(ui.max_rect().right(), ui.max_rect().top()),
                ],
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            // Get a copy of layers to work with
            let layer_manager = project.fractal_settings.layers.lock().unwrap();
            let layers = layer_manager.layers.clone();
            drop(layer_manager);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                for (layer_i, layer) in layers.iter().enumerate() {
                    // Frame containing all mappers for this layer
                    let layer_frame = egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(Self::LAYER_INNER_X_MARGIN as i8, 5))
                        .outer_margin(egui::Margin::symmetric(Self::LAYER_OUTER_X_MARGIN as i8, 0))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                        .corner_radius(5.0);

                    let frame_height = Self::compute_layer_height(ui, project, layer_i);

                    layer_frame.show(ui, |ui| {
                        ui.set_min_height(frame_height);
                        ui.set_max_height(frame_height);

                        ui.set_min_width(ui.available_width());
                        ui.set_max_width(ui.available_width());

                        ui.horizontal_centered(|ui| {
                            ui.label(&layer.name);

                            let add_mapper_width = ui.available_width();

                            // Edit mappers
                            ui.vertical(|ui| {
                                let mut delete_mapper_index = None;
                                for (mapper_i, mapper) in project
                                    .audio_mapper
                                    .layer_mappings_mut(layer_i)
                                    .iter_mut()
                                    .enumerate()
                                {
                                    // Edit individual mapper
                                    ui.horizontal(|ui| {
                                        changed |=
                                            Self::feature_mapper(ui, mapper, mapper_i, layer_i);

                                        // Delete mapper button
                                        if ui
                                            .add_sized(
                                                [Self::MAPPING_DELETE_WIDTH, ui.available_height()],
                                                egui::Button::new("x"),
                                            )
                                            .clicked()
                                        {
                                            delete_mapper_index = Some(mapper_i);
                                        }
                                    });
                                }

                                if let Some(index) = delete_mapper_index {
                                    project
                                        .audio_mapper
                                        .layer_mappings_mut(layer_i)
                                        .remove(index);
                                    changed = true;
                                }

                                // Add new mapper button
                                if ui
                                    .add_sized(
                                        [add_mapper_width, Self::MAPPING_PARAM_HEIGHT],
                                        egui::Button::new("+").fill(crate::ui::DARK_ACCENT_COLOUR),
                                    )
                                    .clicked()
                                {
                                    project
                                        .audio_mapper
                                        .layer_mappings_mut(layer_i)
                                        .push(MapperType::default());
                                    changed = true;
                                }
                            });
                        });
                    });
                }
            });
        });

        changed
    }

    fn compute_layer_height(
        ui: &egui::Ui,
        project: &mut crate::project::Project,
        layer_i: usize,
    ) -> f32 {
        // layer height depends on the number of mappings for each mapper of that layer
        let mut height = Self::MAPPING_PARAM_HEIGHT;
        for mapper in project.audio_mapper.layer_mappings_mut(layer_i) {
            height +=
                Self::compute_mapping_frame_height(ui, mapper) + 2.0 * ui.spacing().item_spacing.y;
        }
        height
    }

    fn compute_mapping_frame_height(ui: &egui::Ui, mapper: &MapperType) -> f32 {
        ((mapper.get_mapper().n_mappings() + 1) as f32
            * (Self::MAPPING_PARAM_HEIGHT + ui.spacing().item_spacing.y))
            .max(Self::MIN_MAPPING_HEIGHT)
    }

    fn feature_mapper(
        ui: &mut egui::Ui,
        mapper: &mut MapperType,
        mapper_i: usize,
        layer_i: usize,
    ) -> bool {
        let mut changed = false;

        let frame = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(5, 5))
            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
            .corner_radius(5.0);

        frame.show(ui, |ui| {
            // ui.set_min_height(Self::MIN_MAPPING_HEIGHT);
            ui.set_min_height(Self::compute_mapping_frame_height(ui, mapper));
            ui.set_min_width(
                ui.available_width() - Self::MAPPING_DELETE_WIDTH - crate::ui::NORMAL_TEXT_SIZE,
            );
            ui.set_max_width(
                ui.available_width() - Self::MAPPING_DELETE_WIDTH - crate::ui::NORMAL_TEXT_SIZE,
            );

            ui.horizontal_centered(|ui| {
                // Mapper type dropdown and parameters
                ui.vertical(|ui| {
                    ui.set_min_width(Self::MAPPING_WIDTH);

                    // dropdown for mapper type. if changed, keep the same layer actions
                    let old_actions = mapper.get_mapper().clone_layer_actions();
                    let changed_type = crate::ui::show_dropdown(
                        ui,
                        mapper,
                        &format!("Mapper Type {}/{}", layer_i, mapper_i),
                    );
                    if changed_type {
                        mapper.get_mapper_mut().set_layer_actions(old_actions);
                    }

                    // mapper type parameters
                    if let Some(on_beat_mapper) = mapper.get_on_beat_mapper_mut() {
                        changed |= Self::on_beat_onset_editor(ui, on_beat_mapper);
                    }
                    if let Some(continuous_sample_mapper) =
                        mapper.get_continuous_sample_mapper_mut()
                    {
                        changed |= Self::feature_window_editor(
                            ui,
                            &format!("Mapper {}/{}", layer_i, mapper_i),
                            &mut continuous_sample_mapper.window,
                        );
                        changed |= Self::intensity_emphasis_editor(
                            ui,
                            &format!("Mapper {}/{}", layer_i, mapper_i),
                            &mut continuous_sample_mapper.intensity_emphasis,
                        );
                    }
                });

                ui.separator();

                // Layer actions for this mapper
                ui.vertical(|ui| {
                    let total_width = ui.available_width();

                    let mut remove_action_index = None;
                    for (action_i, action) in mapper
                        .get_mapper_mut()
                        .layer_actions_mut()
                        .iter_mut()
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            crate::ui::show_dropdown(
                                ui,
                                *action,
                                format!("Layer Action {}/{}/{}", layer_i, mapper_i, action_i),
                            );

                            let value_changed = action.edit_inner_value(ui);
                            if value_changed && action.timeline_needs_recompute() {
                                changed = true;
                            }

                            ui.add_space(ui.available_width() - crate::ui::NORMAL_TEXT_SIZE);

                            if ui.button("x").clicked() {
                                remove_action_index = Some(action_i);
                            }
                        });
                    }

                    if let Some(index) = remove_action_index {
                        let removed_action = mapper.get_mapper_mut().remove_action(index);
                        if let Some(removed_action) = removed_action {
                            if removed_action.timeline_needs_recompute() {
                                changed = true;
                            }
                        }
                    }

                    // Add new action button
                    if ui
                        .add_sized(
                            [total_width, Self::MAPPING_PARAM_HEIGHT],
                            egui::Button::new("+"),
                        )
                        .clicked()
                    {
                        mapper.get_mapper_mut().add_default_mapping();
                    }
                });
            });
        });

        changed
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

            ui.label("Window type:").on_hover_text(
                "Position of the window relative to the current timestamp of the frame.",
            );
            changed |= crate::ui::show_dropdown(
                ui,
                &mut window.window_type,
                format!("{}_window_type_dropdown", heading.to_lowercase()),
            );
        });

        changed
    }

    /// Renders an intensity emphasis editor for the given intensity emphasis
    ///
    /// Returns true if any changes were made to the zoom timeline
    fn intensity_emphasis_editor(
        ui: &mut egui::Ui,
        heading: &str,
        intensity_emphasis: &mut IntensityEmphasis,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Intensity emphasis:").on_hover_text(
                "Whether to emphasize high or low intensity moments for this feature",
            );
            changed |= crate::ui::show_dropdown(
                ui,
                intensity_emphasis,
                format!("{}_intensity_emphasis", heading),
            );

            match intensity_emphasis {
                IntensityEmphasis::Equal => {}
                IntensityEmphasis::High(n) | IntensityEmphasis::Low(n) => {
                    ui.label("Strength:");
                    let response = ui.add(egui::DragValue::new(n).speed(0.1).range(1.0..=f32::MAX));
                    if response.changed() {
                        changed = true;
                    }
                }
            }
        });

        changed
    }

    /// Renders an on-beat mapper editor for the given on-beat mapper
    ///
    /// Returns true if any changes were made to the zoom timeline
    fn on_beat_onset_editor(ui: &mut egui::Ui, on_beat_mapper: &mut OnBeatMapper) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Attack duration:").on_hover_text(
                "The duration before beats over which the 'on beat' effect will ramp up",
            );
            let attack_response = ui.add(
                egui::DragValue::new(&mut on_beat_mapper.attack_duration)
                    .speed(0.01)
                    .range(0.0..=1.0),
            );
            if attack_response.changed() {
                changed = true;
            }

            ui.add_space(10.0);
            ui.label("Decay duration:").on_hover_text(
                "The duration after beats over which the 'on beat' effect will ramp down",
            );
            let decay_response = ui.add(
                egui::DragValue::new(&mut on_beat_mapper.decay_duration)
                    .speed(0.01)
                    .range(0.0..=1.0),
            );
            if decay_response.changed() {
                changed = true;
            }
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
