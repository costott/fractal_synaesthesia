use egui::{ImageButton, load::SizedTexture};

use crate::{
    rendering::{
        manager::layer::LayerAlgorithmKind,
        orbit_trap::{OrbitTrap, OrbitTrapType},
    },
    types::ComplexNumber,
    ui::{
        menus::fractal_settings::layers_editor::palette_editor::PaletteEditor, window::WindowParams,
    },
};

pub struct LayerSettings {
    params: WindowParams,

    /// Whether to update the local specifics copy on the next update. This is set to true when the layer algorithm changes, and set to false when the local specifics copy is updated.
    pub update_locals: bool,

    palette_texture: Option<egui::TextureHandle>,
    local_copy: Option<LocalCopy>,

    palette_editor: PaletteEditor,
}
impl LayerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            update_locals: true,
            palette_texture: None,
            local_copy: None,
            palette_editor: PaletteEditor::new(params),
        }
    }

    pub fn selected_layer_changed(&mut self) {
        self.update_locals = true;
        self.palette_editor.close();
    }

    /// Returns whether the layer has changed
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
        selected_layer: usize,
    ) -> bool {
        if self.palette_editor.is_open() {
            let palette_changed = self.palette_editor.update(egui_ctx, project, ctx);
            if palette_changed {
                self.palette_texture = None;
            }
            return palette_changed;
        }

        let layer = &mut project.fractal_settings.layers.lock().unwrap().layers[selected_layer];
        let mut layer_changed = false;

        if self.update_locals {
            self.local_copy = Some(LocalCopy::copy_from_algorithm(&layer.algorithm));
            self.update_locals = false;
            self.palette_texture = None;
        }

        if self.palette_texture.is_none() {
            let palette = layer.palette.get_full_gradient(100.0, 100.0 * (9. / 16.));
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [palette.width() as usize, palette.height() as usize],
                &palette.get_texture_data().bytes,
            );
            self.palette_texture =
                Some(egui_ctx.load_texture("palette", color_image, egui::TextureOptions::NEAREST));
        }

        self.params.sized_area("layer_settings", egui_ctx, |ui| {
            ui.add_space(10.0);
            ui.label(egui::RichText::new(layer.name.clone()).heading().strong());
            egui::Grid::new("layer_settings_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    // Formula
                    ui.label(
                        egui::RichText::new("Formula")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    if crate::ui::show_dropdown(ui, &mut layer.algorithm, "formula") {
                        layer_changed = true;
                        self.update_locals = true;
                    }

                    ui.end_row();

                    // Palette
                    ui.label(
                        egui::RichText::new("Palette")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    let t =
                        SizedTexture::from_handle(&self.palette_texture.as_ref().unwrap().clone());
                    let image = egui::Image::from_texture(t);
                    let response = ui.add(ImageButton::new(image));
                    if response.clicked() {
                        self.palette_editor.open(selected_layer);
                    }

                    ui.end_row();

                    // Application range
                    ui.label(
                        egui::RichText::new("Application range")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    if crate::ui::show_dropdown(
                        ui,
                        &mut layer.application_range,
                        "application range",
                    ) {
                        layer_changed = true;
                    }

                    ui.end_row();
                });

            ui.separator();

            egui::Grid::new("specific_layer_settings_grid")
                .num_columns(2)
                .show(ui, |ui| match &mut layer.algorithm {
                    LayerAlgorithmKind::Colour { bailout2 } => {
                        layer_changed |= LayerSettings::colour_settings(
                            ui,
                            ctx.rendering,
                            bailout2,
                            &mut self.local_copy,
                        );
                    }
                    LayerAlgorithmKind::OrbitTrap { bailout2, trap } => {
                        layer_changed |= LayerSettings::orbit_trap_settings(
                            ui,
                            ctx.rendering,
                            bailout2,
                            trap,
                            &mut self.update_locals,
                            &mut self.local_copy,
                        )
                    }
                    LayerAlgorithmKind::Shading3D {
                        bailout2,
                        h2,
                        angle,
                    } => {
                        layer_changed |= LayerSettings::shading3d_settings(
                            ui,
                            ctx.rendering,
                            bailout2,
                            h2,
                            angle,
                            &mut self.local_copy,
                        );
                    }
                    LayerAlgorithmKind::TriangleInequality { bailout2, apower } => {
                        layer_changed |= LayerSettings::triangle_inequality_settings(
                            ui,
                            ctx.rendering,
                            bailout2,
                            apower,
                            &mut self.local_copy,
                        );
                    }
                    LayerAlgorithmKind::StripeAverage {
                        bailout2,
                        skip_iteration,
                        stripe_density,
                    } => {
                        layer_changed |= LayerSettings::stripe_average_settings(
                            ui,
                            ctx.rendering,
                            bailout2,
                            skip_iteration,
                            stripe_density,
                            project
                                .fractal_settings
                                .params
                                .lock()
                                .unwrap()
                                .max_iterations,
                            &mut self.local_copy,
                        );
                    }
                });
        });

        layer_changed
    }

    fn colour_settings(
        ui: &mut egui::Ui,
        ctx_rendering: bool,
        bailout2: &mut f64,
        local_copy: &mut Option<LocalCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Colour Settings").heading());
        ui.end_row();

        if let Some(local) = local_copy.as_mut() {
            let bailout2_str = bailout2.sqrt().to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Bailout",
                &mut local.bailout2,
                ctx_rendering,
                || bailout2_str.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *bailout2 = val * val;
                    }
                },
            );
        }

        layers_changed
    }

    fn orbit_trap_settings(
        ui: &mut egui::Ui,
        ctx_rendering: bool,
        bailout2: &mut f64,
        trap: &mut OrbitTrapType,
        update_locals: &mut bool,
        local_copy: &mut Option<LocalCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Orbit Trap Settings").heading());
        ui.end_row();

        ui.label(
            egui::RichText::new("Type")
                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
        );

        if crate::ui::show_dropdown(ui, trap, "orbit trap type") {
            layers_changed = true;
            *update_locals = true;
        }

        ui.end_row();

        ui.label(
            egui::RichText::new("Analysis Type")
                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
        );

        if crate::ui::show_dropdown(ui, trap.get_analysis_mut(), "orbit trap analysis") {
            layers_changed = true;
        }

        ui.end_row();

        match trap {
            OrbitTrapType::Point(point) => {
                if let Some(LocalCopy {
                    bailout2: _,
                    specifics:
                        Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Point {
                            center_re: local_center_re,
                            center_im: local_center_im,
                        })),
                }) = local_copy.as_mut()
                {
                    let centre_real = point.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        local_center_re,
                        ctx_rendering,
                        || centre_real.clone(),
                        |new| point.center.update_real_from_string(new),
                    );

                    let centre_im = point.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        local_center_im,
                        ctx_rendering,
                        || centre_im.clone(),
                        |new| point.center.update_im_from_string(new),
                    );
                }
            }
            OrbitTrapType::Cross(cross) => {
                if let Some(LocalCopy {
                    bailout2: _,
                    specifics:
                        Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Cross {
                            center_re: local_center_re,
                            center_im: local_center_im,
                            arm_length: local_arm_length,
                        })),
                }) = local_copy.as_mut()
                {
                    let centre_real = cross.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        local_center_re,
                        ctx_rendering,
                        || centre_real.clone(),
                        |new| cross.center.update_real_from_string(new),
                    );

                    let centre_im = cross.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        local_center_im,
                        ctx_rendering,
                        || centre_im.clone(),
                        |new| cross.center.update_im_from_string(new),
                    );

                    let arm = cross.arm_length.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Arm length",
                        local_arm_length,
                        ctx_rendering,
                        || arm.clone(),
                        |new| {
                            if let Ok(val) = new.parse::<f64>() {
                                cross.arm_length = val;
                            }
                        },
                    );
                }
            }
            OrbitTrapType::Circle(circle) => {
                if let Some(LocalCopy {
                    bailout2: _,
                    specifics:
                        Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Circle {
                            center_re: local_center_re,
                            center_im: local_center_im,
                            radius: local_radius,
                        })),
                }) = local_copy.as_mut()
                {
                    let centre_real = circle.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        local_center_re,
                        ctx_rendering,
                        || centre_real.clone(),
                        |new| circle.center.update_real_from_string(new),
                    );

                    let centre_im = circle.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        local_center_im,
                        ctx_rendering,
                        || centre_im.clone(),
                        |new| circle.center.update_im_from_string(new),
                    );

                    let c_radius = circle.radius.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Radius",
                        local_radius,
                        ctx_rendering,
                        || c_radius.clone(),
                        |new| {
                            if let Ok(val) = new.parse::<f64>() {
                                circle.radius = val;
                            }
                        },
                    );
                }
            }
        };

        // edit bailout
        if let Some(LocalCopy {
            bailout2: local_bailout2,
            specifics: _,
        }) = local_copy.as_mut()
        {
            let bailout2_str = bailout2.sqrt().to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Bailout",
                local_bailout2,
                ctx_rendering,
                || bailout2_str.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *bailout2 = val * val;
                    }
                },
            );
        }

        layers_changed
    }

    fn shading3d_settings(
        ui: &mut egui::Ui,
        ctx_rendering: bool,
        bailout2: &mut f64,
        h2: &mut f64,
        angle: &mut f64,
        local_copy: &mut Option<LocalCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Shading 3D Settings").heading());
        ui.end_row();

        if let Some(LocalCopy {
            bailout2: local_bailout2,
            specifics:
                Some(LocalSpecificsCopy::Shading3D {
                    h2: local_h2,
                    angle: _,
                }),
        }) = local_copy.as_mut()
        {
            let curr_h2 = h2.to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Height",
                local_h2,
                ctx_rendering,
                || curr_h2.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *h2 = val;
                    }
                },
            );

            ui.label(
                egui::RichText::new("Angle")
                    .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
            );

            let response = ui.add(egui::Slider::new(angle, 0.0..=360.0));
            if response.changed() {
                layers_changed = true;
            }

            ui.end_row();

            let bailout2_str = bailout2.sqrt().to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Bailout",
                local_bailout2,
                ctx_rendering,
                || bailout2_str.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *bailout2 = val * val;
                    }
                },
            );
        }

        layers_changed
    }

    fn triangle_inequality_settings(
        ui: &mut egui::Ui,
        ctx_rendering: bool,
        bailout2: &mut f64,
        apower: &mut f64,
        local_copy: &mut Option<LocalCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Triangle Inequality Settings").heading());
        ui.end_row();

        if let Some(LocalCopy {
            bailout2: local_bailout2,
            specifics:
                Some(LocalSpecificsCopy::TriangleInequality {
                    apower: local_apower,
                }),
        }) = local_copy.as_mut()
        {
            let curr_apower = apower.to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Apower",
                local_apower,
                ctx_rendering,
                || curr_apower.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *apower = val;
                    }
                },
            );

            ui.end_row();

            let bailout2_str = bailout2.sqrt().to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Bailout",
                local_bailout2,
                ctx_rendering,
                || bailout2_str.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *bailout2 = val * val;
                    }
                },
            );
        }

        layers_changed
    }

    fn stripe_average_settings(
        ui: &mut egui::Ui,
        ctx_rendering: bool,
        bailout2: &mut f64,
        skip_iteration: &mut u32,
        stripe_density: &mut f64,
        max_iterations: u32,
        local_copy: &mut Option<LocalCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Stripe Average Settings").heading());
        ui.end_row();

        if let Some(LocalCopy {
            bailout2: local_bailout2,
            specifics:
                Some(LocalSpecificsCopy::StripeAverage {
                    skip_iteration: _,
                    stripe_density: _,
                }),
        }) = local_copy.as_mut()
        {
            ui.label(
                egui::RichText::new("Skip iteration")
                    .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
            );

            let response = ui.add(egui::Slider::new(skip_iteration, 1..=max_iterations - 1));
            if response.changed() {
                layers_changed = true;
            }

            ui.end_row();

            ui.label(
                egui::RichText::new("Stripe Density")
                    .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
            );

            let response = ui.add(
                egui::DragValue::new(stripe_density)
                    .speed(1.0)
                    .range(0.0..=max_iterations as f64 - 1.0),
            );
            if response.changed() {
                layers_changed = true;
            }

            ui.end_row();

            let bailout2_str = bailout2.sqrt().to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Bailout",
                local_bailout2,
                ctx_rendering,
                || bailout2_str.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *bailout2 = val * val;
                    }
                },
            );
        }

        layers_changed
    }
}

struct LocalCopy {
    bailout2: String,
    specifics: Option<LocalSpecificsCopy>,
}
impl LocalCopy {
    fn copy_from_algorithm(algorithm: &LayerAlgorithmKind) -> Self {
        let bailout2 = match algorithm {
            LayerAlgorithmKind::Colour { bailout2 } => *bailout2,
            LayerAlgorithmKind::OrbitTrap { bailout2, .. } => *bailout2,
            LayerAlgorithmKind::Shading3D { bailout2, .. } => *bailout2,
            LayerAlgorithmKind::TriangleInequality { bailout2, .. } => *bailout2,
            LayerAlgorithmKind::StripeAverage { bailout2, .. } => *bailout2,
        }
        .to_string();

        Self {
            bailout2,
            specifics: LocalSpecificsCopy::copy_from_algorithm(algorithm),
        }
    }
}

enum LocalSpecificsCopy {
    Colour,
    OrbitTrap(LocalOrbitTrapCopy),
    Shading3D {
        h2: String,
        angle: String,
    },
    TriangleInequality {
        apower: String,
    },
    StripeAverage {
        skip_iteration: String,
        stripe_density: String,
    },
}
impl LocalSpecificsCopy {
    fn copy_from_algorithm(algorithm: &LayerAlgorithmKind) -> Option<Self> {
        match algorithm {
            LayerAlgorithmKind::Colour { .. } => None,
            LayerAlgorithmKind::OrbitTrap { trap, .. } => {
                Some(Self::OrbitTrap(LocalOrbitTrapCopy::copy_from_trap(trap)))
            }
            LayerAlgorithmKind::Shading3D { h2, angle, .. } => Some(Self::Shading3D {
                h2: h2.to_string(),
                angle: angle.to_string(),
            }),
            LayerAlgorithmKind::TriangleInequality { apower, .. } => {
                Some(Self::TriangleInequality {
                    apower: apower.to_string(),
                })
            }
            LayerAlgorithmKind::StripeAverage {
                skip_iteration,
                stripe_density,
                ..
            } => Some(Self::StripeAverage {
                skip_iteration: skip_iteration.to_string(),
                stripe_density: stripe_density.to_string(),
            }),
        }
    }
}

enum LocalOrbitTrapCopy {
    Point {
        center_re: String,
        center_im: String,
    },
    Cross {
        center_re: String,
        center_im: String,
        arm_length: String,
    },
    Circle {
        center_re: String,
        center_im: String,
        radius: String,
    },
}
impl LocalOrbitTrapCopy {
    fn copy_from_trap(trap: &OrbitTrapType) -> Self {
        match trap {
            OrbitTrapType::Point(point) => Self::Point {
                center_re: point.center.real.to_string(),
                center_im: point.center.im.to_string(),
            },
            OrbitTrapType::Cross(cross) => Self::Cross {
                center_re: cross.center.real.to_string(),
                center_im: cross.center.im.to_string(),
                arm_length: cross.arm_length.to_string(),
            },
            OrbitTrapType::Circle(circle) => Self::Circle {
                center_re: circle.center.real.to_string(),
                center_im: circle.center.im.to_string(),
                radius: circle.radius.to_string(),
            },
        }
    }
}
