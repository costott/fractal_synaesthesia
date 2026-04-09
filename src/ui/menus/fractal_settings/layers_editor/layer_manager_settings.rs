use std::sync::{Arc, Mutex};

use egui::load::SizedTexture;

use crate::{
    rendering::{
        fractal_visualiser::FractalVisualiser,
        manager::{
            layer::{Layer, LayerMappingKind},
            layer_manager::LayerManager,
        },
        palette::Palette,
    },
    types::colour::*,
    ui::{fractal::fractal_canvas::CanvasDimensions, window::WindowParams},
};

pub struct LayerManagerSettings {
    params: WindowParams,

    dragging_index: Option<usize>,
    drop_target_index: Option<usize>,

    layer_previews: Vec<Option<egui::TextureHandle>>,
}
impl LayerManagerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            dragging_index: None,
            drop_target_index: None,
            layer_previews: vec![],
        }
    }

    pub fn layer_changed(&mut self, layer_idx: usize) {
        self.layer_previews[layer_idx] = None;
    }

    /// Returns whether the selected layer changed
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
        selected_layer: &mut usize,
    ) -> bool {
        let layer_manager = &mut project.fractal_settings.layers.lock().unwrap();

        if layer_manager.layers.len() < self.layer_previews.len() {
            self.layer_previews.truncate(layer_manager.layers.len());
        }

        // Set length of layer previews
        while self.layer_previews.len() < layer_manager.layers.len() {
            self.layer_previews.push(None);
        }

        // Render any previews that need updating
        for (layer_idx, layer_preview) in self.layer_previews.iter_mut().enumerate() {
            if ctx.rendering {
                continue;
            }

            if !ctx.update_previews && layer_preview.is_some() {
                continue;
            }

            let mut layer_clone = layer_manager.layers[layer_idx].clone();
            layer_clone.strength = 1.0;
            let layers_vector = if layer_clone.algorithm.get_mapping_kind()
                == LayerMappingKind::Shade
            {
                let to_shade_layer = Layer::new(
                        crate::rendering::manager::layer::LayerAlgorithmKind::Colour { bailout2: crate::rendering::algorithms::layer_algorithms::ColourAlgorithm::DEFAULT_BAILOUT2 },
                        crate::rendering::manager::layer::LayerRange::Both,
                        1.0,
                        Palette::new_even(
                            vec![WHITE, WHITE],
                            crate::rendering::palette::PaletteMappingType::Constant,
                            1.0,
                            0.0,
                        ),
                    );
                vec![to_shade_layer, layer_clone]
            } else {
                vec![layer_clone]
            };

            let mut fractal_params = project.fractal_settings.params.lock().unwrap().clone();
            let canvas_dims =
                CanvasDimensions::new_from_aspect_with_width(ctx.fractal_dims.aspect_ratio(), 60);
            let width_proportion = ctx.fractal_dims.width as f64 / canvas_dims.width as f64;
            fractal_params.pixel_step *= width_proportion;

            let mut tmp_visualiser = FractalVisualiser::new(
                project.fractal_settings.params.clone(),
                canvas_dims,
                Arc::new(Mutex::new(LayerManager::new(layers_vector, false))),
                1,
                false,
            );

            tmp_visualiser.update_render(Arc::new(Mutex::new(fractal_params)));

            // Wait for render to be complete
            while !tmp_visualiser.finished_render() {}

            let rendered_image = tmp_visualiser.rendered_image();
            let rendered_image = rendered_image.lock().unwrap();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [
                    rendered_image.width() as usize,
                    rendered_image.height() as usize,
                ],
                &rendered_image.bytes,
            );
            *layer_preview = Some(egui_ctx.load_texture(
                format!("preview_layer_{layer_idx}"),
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }
        if !ctx.rendering {
            ctx.update_previews = false;
        }

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

                // Add and delete buttons
                let mut changed_layer_num = false;
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    if ui.button("+").clicked() && !ctx.rendering {
                        layer_manager.add_layer();
                        changed_layer_num = true;
                    }

                    ui.add_space(ui.available_width() - 25.0);

                    if ui.button("🗑").clicked() {
                        if layer_manager.layers.len() == 1 {
                            return;
                        }

                        layer_manager.remove_layer(*selected_layer);
                        self.layer_previews.remove(*selected_layer);
                        *selected_layer = 0;
                        changed_layer_num = true;
                        selected_layer_changed = true;
                    }
                });
                if changed_layer_num {
                    ctx.update_layers = true;
                    ctx.request_render = true;
                    return;
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
                                    // Preview
                                    if let Some(preview) = &self.layer_previews[layer_idx] {
                                        let t = SizedTexture::from_handle(preview);

                                        let image = egui::Image::from_texture(t);
                                        ui.add(image);
                                    } else {
                                        ui.add_space(60.0 + ui.spacing().item_spacing.x);
                                    }

                                    // Name
                                    ui.add(
                                        egui::TextEdit::singleline(&mut layer.name)
                                            .desired_width(100.0),
                                    );

                                    // Strength
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

                                    // Drag area
                                    let drag_icon = ui.label("...");
                                    let drag_icon_response = ui.interact(
                                        drag_icon.interact_rect,
                                        ui.make_persistent_id(format!("drag_icon_{layer_idx}")),
                                        egui::Sense::click_and_drag(),
                                    );
                                    if drag_icon_response.drag_started() {
                                        self.dragging_index = Some(layer_idx);
                                        self.drop_target_index = None;
                                    }
                                    if drag_icon_response.hovered() {
                                        ui.output_mut(|o| {
                                            o.cursor_icon = egui::CursorIcon::ResizeVertical
                                        });
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

                        let to_move = self.layer_previews.remove(from);
                        self.layer_previews.insert(to_idx, to_move);

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
