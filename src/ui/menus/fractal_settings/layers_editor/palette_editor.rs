use crate::{
    rendering::palette::{PaletteMappingType, color_to_rbga, rgba_to_color},
    ui::{Dropdown as _, window::WindowParams},
};

struct PaletteTextures {
    gradient: egui::TextureHandle,
    full_palette: egui::TextureHandle,
}

struct EditingPalette {
    layer_index: usize,
    textures: Option<PaletteTextures>,
    selected_point: usize,
}

pub struct PaletteEditor {
    params: WindowParams,

    editing_palette: Option<EditingPalette>,

    is_open: bool,
}
impl PaletteEditor {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            editing_palette: None,
            is_open: false,
        }
    }

    pub fn open(&mut self, layer_index: usize) {
        self.is_open = true;
        self.editing_palette = Some(EditingPalette {
            layer_index,
            textures: None,
            selected_point: 0,
        });
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.editing_palette = None;
    }

    /// Returns whether the layer has changed
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut crate::ui::window::WindowContext,
    ) -> bool {
        let mut changed_layer = false;
        let mut need_to_close = false;

        if let Some(editing_palette) = &mut self.editing_palette {
            let mut changed_gradient = false;

            let layer = &mut ctx.layer_manager.lock().unwrap().layers[editing_palette.layer_index];

            if editing_palette.textures.is_none() {
                let width = self.params.width as f32 * 0.7;

                let gradient = layer.palette.get_full_gradient(width, width * (9. / 32.));
                let palette_tex = layer.palette.get_full_palette(width, width * (9. / 64.));

                editing_palette.textures = Some(PaletteTextures {
                    gradient: crate::ui::get_texture_handle_from_texture2d(
                        egui_ctx, gradient, "gradient",
                    ),
                    full_palette: crate::ui::get_texture_handle_from_texture2d(
                        egui_ctx,
                        palette_tex,
                        "full_palette",
                    ),
                });
            }

            self.params.sized_area("Palette_editor", egui_ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!("Editing '{}' palette", layer.name)).heading(),
                );

                // Gradient and droppers
                ui.vertical_centered(|ui| {
                    let (palette_rect, response) = crate::ui::draw_image_from_handle(
                        ui,
                        &editing_palette.textures.as_ref().unwrap().gradient,
                    );
                    // border
                    ui.painter().rect_stroke(
                        palette_rect,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                        egui::StrokeKind::Outside,
                    );

                    if response.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                    }

                    // Add point if palette clicked
                    if response.clicked() {
                        layer.palette.add_point(
                            (response.interact_pointer_pos().unwrap().x - palette_rect.left())
                                / palette_rect.width(),
                        );
                    }

                    // Dropper points
                    for (i, point) in layer.palette.colour_map.get_map().iter_mut().enumerate() {
                        let x = palette_rect.left() + point.percent_pos * palette_rect.width();
                        let top_y = palette_rect.bottom();

                        // let c: [u8; 4] = point.colour.into();
                        // let c32 = egui::Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]);
                        let c = color_to_rbga(point.colour);
                        let bg_colour = if editing_palette.selected_point == i {
                            egui::Color32::GRAY
                        } else {
                            egui::Color32::DARK_GRAY
                        };

                        let triangle_half_width = 9.0;
                        let triangle_height = 12.0;
                        let circle_padding = 0.0;
                        let circle_radius = 8.0;

                        let triangle_points = [
                            egui::pos2(x, top_y),
                            egui::pos2(x - triangle_half_width, top_y + triangle_height),
                            egui::pos2(x + triangle_half_width, top_y + triangle_height),
                        ];
                        ui.painter().add(egui::Shape::convex_polygon(
                            triangle_points.to_vec(),
                            bg_colour,
                            egui::Stroke::NONE,
                        ));

                        ui.painter().rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(x - triangle_half_width, top_y + triangle_height),
                                egui::pos2(
                                    x + triangle_half_width,
                                    top_y + triangle_height + circle_padding + triangle_half_width,
                                ),
                            ),
                            0.0,
                            bg_colour,
                        );

                        ui.painter().circle_filled(
                            egui::pos2(x, top_y + triangle_height + circle_padding + circle_radius),
                            triangle_half_width,
                            bg_colour,
                        );

                        ui.painter().circle_filled(
                            egui::pos2(x, top_y + triangle_height + circle_padding + circle_radius),
                            circle_radius,
                            c,
                        );

                        // Detect dragging
                        let id = ui.make_persistent_id(("colour_point", i));
                        let point_response = ui.interact(
                            egui::Rect::from_min_max(
                                egui::pos2(x - triangle_half_width, top_y),
                                egui::pos2(
                                    x + triangle_half_width,
                                    top_y + triangle_height + circle_padding + circle_radius * 2.,
                                ),
                            ),
                            id,
                            egui::Sense::click_and_drag(),
                        );

                        if point_response.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                        }

                        if point_response.clicked() || point_response.dragged() {
                            editing_palette.selected_point = i;
                        }

                        if point_response.dragged() {
                            let delta_x = point_response.drag_delta().x;
                            let new_x =
                                (x + delta_x).clamp(palette_rect.left(), palette_rect.right());
                            point.percent_pos =
                                (new_x - palette_rect.left()) / palette_rect.width();
                            if delta_x.abs() > 0.1 {
                                // only signal change if it was significant
                                changed_gradient = true;
                                changed_layer = true;
                            }
                        }
                    }

                    ui.add_space(40.0);

                    // Colour picker
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Edit selected point")
                                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                        );

                        let selected_point =
                            &mut layer.palette.colour_map.get_map()[editing_palette.selected_point];
                        let mut rgba = color_to_rbga(selected_point.colour);
                        let response = egui::color_picker::color_edit_button_rgba(
                            ui,
                            &mut rgba,
                            egui::color_picker::Alpha::OnlyBlend,
                        );
                        if response.changed() {
                            selected_point.colour = rgba_to_color(rgba);
                            changed_gradient = true;
                            changed_layer = true;
                        }
                    });
                });

                ui.separator();

                // Palette and macro palette settings
                ui.vertical_centered(|ui| {
                    let (palette_rect, _) = crate::ui::draw_image_from_handle(
                        ui,
                        &editing_palette.textures.as_ref().unwrap().full_palette,
                    );
                    ui.painter().rect_stroke(
                        palette_rect,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                        egui::StrokeKind::Outside,
                    );

                    // Mapping type
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Mapping type")
                                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                        );
                        if crate::ui::show_dropdown(
                            ui,
                            &mut layer.palette.mapping_type,
                            "palette mapping type",
                        ) {
                            changed_gradient = true;
                            changed_layer = true;
                        }
                    });

                    // Length
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Length")
                                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                        );

                        if ui
                            .add(egui::Slider::new(
                                layer.palette.get_length_mut(),
                                f32::MIN_POSITIVE..=1.0,
                            ))
                            .changed()
                        {
                            changed_gradient = true;
                            changed_layer = true;
                        }
                    });

                    // Offset
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Offset")
                                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                        );

                        if ui
                            .add(egui::Slider::new(
                                layer.palette.get_offset_mut(),
                                f32::MIN_POSITIVE..=1.0,
                            ))
                            .changed()
                        {
                            changed_gradient = true;
                            changed_layer = true;
                        }
                    });

                    if ui.button("Close").clicked() {
                        need_to_close = true;
                    }
                });
            });

            if changed_gradient {
                editing_palette.textures = None;
            }
        }

        if need_to_close {
            self.close();
        }

        changed_layer
    }
}
