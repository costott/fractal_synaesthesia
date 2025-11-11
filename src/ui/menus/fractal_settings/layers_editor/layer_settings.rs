use egui::{ImageButton, load::SizedTexture};

use crate::{
    rendering::{
        manager::layer::{LayerAlgorithmKind, LayerRange},
        orbit_trap::{OrbitTrap, OrbitTrapAnalysis, OrbitTrapType},
    },
    types::ComplexNumber,
    ui::{
        Dropdown,
        menus::fractal_settings::layers_editor::palette_editor::PaletteEditor,
        window::{Window, WindowParams},
    },
};

pub struct LayerSettings {
    params: WindowParams,

    pub update_locals: bool,

    palette_texture: Option<egui::TextureHandle>,
    local_specifics_copy: Option<LocalSpecificsCopy>,

    palette_editor: PaletteEditor,
}
impl LayerSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            update_locals: true,
            palette_texture: None,
            local_specifics_copy: None,
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
        ctx: &mut crate::ui::window::WindowContext,
        selected_layer: usize,
    ) -> bool {
        if self.palette_editor.is_open() {
            return self.palette_editor.update(egui_ctx, ctx);
        }

        let layer = &mut ctx.layer_manager.lock().unwrap().layers[selected_layer];
        let mut layer_changed = false;

        if self.update_locals {
            self.local_specifics_copy = LocalSpecificsCopy::copy_from_algoritm(&layer.algorithm);
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
            ui.label(egui::RichText::new(layer.name.clone()).heading());
            egui::Grid::new("layer_settings_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Formula")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    if crate::ui::show_dropdown(ui, &mut layer.algorithm, "formula") {
                        layer_changed = true;
                        self.update_locals = true;
                    }

                    ui.end_row();

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
                    LayerAlgorithmKind::Colour => {}
                    LayerAlgorithmKind::OrbitTrap { trap } => {
                        layer_changed |= LayerSettings::orbit_trap_settings(
                            ui,
                            trap,
                            &mut self.update_locals,
                            &mut self.local_specifics_copy,
                        )
                    }
                    LayerAlgorithmKind::Shading3D { h2, angle } => {
                        layer_changed |= LayerSettings::shading3d_settings(
                            ui,
                            h2,
                            angle,
                            &mut self.local_specifics_copy,
                        );
                    }
                    LayerAlgorithmKind::TriangleInequality { apower } => {
                        layer_changed |= LayerSettings::triangle_inequality_settings(
                            ui,
                            apower,
                            &mut self.local_specifics_copy,
                        );
                    }
                    LayerAlgorithmKind::StripeAverage {
                        skip_iteration,
                        stripe_density,
                    } => {
                        layer_changed |= LayerSettings::stripe_average_settings(
                            ui,
                            skip_iteration,
                            stripe_density,
                            ctx.fractal_params.lock().unwrap().max_iterations,
                            &mut self.local_specifics_copy,
                        );
                    }
                });
        });

        layer_changed
    }

    fn orbit_trap_settings(
        ui: &mut egui::Ui,
        trap: &mut OrbitTrapType,
        update_locals: &mut bool,
        local_specifics_copy: &mut Option<LocalSpecificsCopy>,
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
                if let Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Point {
                    center_re,
                    center_im,
                })) = local_specifics_copy.as_mut()
                {
                    let centre_real = point.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        center_re,
                        || centre_real.clone(),
                        |new| point.center.update_real_from_string(new),
                    );

                    let centre_im = point.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        center_im,
                        || centre_im.clone(),
                        |new| point.center.update_im_from_string(new),
                    );
                }
            }
            OrbitTrapType::Cross(cross) => {
                if let Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Cross {
                    center_re,
                    center_im,
                    arm_length,
                })) = local_specifics_copy.as_mut()
                {
                    let centre_real = cross.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        center_re,
                        || centre_real.clone(),
                        |new| cross.center.update_real_from_string(new),
                    );

                    let centre_im = cross.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        center_im,
                        || centre_im.clone(),
                        |new| cross.center.update_im_from_string(new),
                    );

                    let arm = cross.arm_length.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Arm length",
                        arm_length,
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
                if let Some(LocalSpecificsCopy::OrbitTrap(LocalOrbitTrapCopy::Circle {
                    center_re,
                    center_im,
                    radius,
                })) = local_specifics_copy.as_mut()
                {
                    let centre_real = circle.center.real.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        center_re,
                        || centre_real.clone(),
                        |new| circle.center.update_real_from_string(new),
                    );

                    let centre_im = circle.center.im.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        center_im,
                        || centre_im.clone(),
                        |new| circle.center.update_im_from_string(new),
                    );

                    let c_radius = circle.radius.to_string();
                    layers_changed |= crate::ui::text_param(
                        ui,
                        "Radius",
                        radius,
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

        layers_changed
    }

    fn shading3d_settings(
        ui: &mut egui::Ui,
        h2: &mut f64,
        angle: &mut f64,
        local_specifics_copy: &mut Option<LocalSpecificsCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Shading 3D Settings").heading());
        ui.end_row();

        if let Some(LocalSpecificsCopy::Shading3D {
            h2: local_h2,
            angle: _,
        }) = local_specifics_copy.as_mut()
        {
            let curr_h2 = h2.to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Height",
                local_h2,
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
        }

        layers_changed
    }

    fn triangle_inequality_settings(
        ui: &mut egui::Ui,
        apower: &mut f64,
        local_specifics_copy: &mut Option<LocalSpecificsCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Triangle Inequality Settings").heading());
        ui.end_row();

        if let Some(LocalSpecificsCopy::TriangleInequality {
            apower: local_apower,
        }) = local_specifics_copy.as_mut()
        {
            let curr_apower = apower.to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Apower",
                local_apower,
                || curr_apower.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *apower = val;
                    }
                },
            );
        }

        layers_changed
    }

    fn stripe_average_settings(
        ui: &mut egui::Ui,
        skip_iteration: &mut u32,
        stripe_density: &mut f64,
        max_iterations: u32,
        local_specifics_copy: &mut Option<LocalSpecificsCopy>,
    ) -> bool {
        let mut layers_changed = false;

        ui.label(egui::RichText::new("Stripe Average Settings").heading());
        ui.end_row();

        if let Some(LocalSpecificsCopy::StripeAverage {
            skip_iteration: _,
            stripe_density: local_stripe_density,
        }) = local_specifics_copy.as_mut()
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

            let curr_density = stripe_density.to_string();
            layers_changed |= crate::ui::text_param(
                ui,
                "Sripe Density",
                local_stripe_density,
                || curr_density.clone(),
                |new| {
                    if let Ok(val) = new.parse::<f64>() {
                        *stripe_density = val;
                    }
                },
            );
        }

        layers_changed
    }
}

enum LocalSpecificsCopy {
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
    fn copy_from_algoritm(algorithm: &LayerAlgorithmKind) -> Option<Self> {
        match algorithm {
            LayerAlgorithmKind::Colour => None,
            LayerAlgorithmKind::OrbitTrap { trap } => {
                Some(Self::OrbitTrap(LocalOrbitTrapCopy::copy_from_trap(trap)))
            }
            LayerAlgorithmKind::Shading3D { h2, angle } => Some(Self::Shading3D {
                h2: h2.to_string(),
                angle: angle.to_string(),
            }),
            LayerAlgorithmKind::TriangleInequality { apower } => Some(Self::TriangleInequality {
                apower: apower.to_string(),
            }),
            LayerAlgorithmKind::StripeAverage {
                skip_iteration,
                stripe_density,
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
