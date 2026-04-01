use crate::ui::{menus::fractal_settings::FractalSettingsWindow, window::WindowParams};

pub struct FractalSettingsFooter {
    pub params: WindowParams,
}
impl FractalSettingsFooter {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }
}
impl FractalSettingsWindow for FractalSettingsFooter {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        _project: &mut crate::project::Project,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) {
        self.params
            .sized_area("Fractal settings footer", egui_ctx, |ui| {
                ui.painter().line_segment(
                    [
                        egui::Pos2::new(ui.max_rect().left(), ui.max_rect().top()),
                        egui::Pos2::new(ui.max_rect().right(), ui.max_rect().top()),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::BLACK),
                );

                ui.horizontal_centered(|ui| {
                    ui.add_space(505.0);

                    match ctx.zoom_window_state {
                        crate::ui::menus::fractal_settings::ZoomWindowInteractState::Inactive {
                            has_history,
                        } => {
                            key_tip(ui, "🖱", "Click and drag to zoom in");
                            if has_history {
                                key_tip(ui, "RMB", "Zoom out");
                            }
                        }
                        crate::ui::menus::fractal_settings::ZoomWindowInteractState::Creating {
                            ..
                        } => {
                            key_tip(ui, "↩", "Apply zoom");
                            key_tip(ui, "🖱", "Resize & Rotate");
                            key_tip(ui, "ESC", "Cancel zoom");
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(5.0);
                        ui.label("Fractal Synaesthesia");
                    });
                });
            });
    }
}

fn key_tip(ui: &mut egui::Ui, key: &str, description: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;

        // wrap key in a rounded rectangle with light gray background
        egui::Frame::new()
            .fill(egui::Color32::LIGHT_GRAY)
            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                ui.add_space(4.0);
                if key.chars().count() == 1 {
                    ui.label(egui::RichText::new(key));
                } else {
                    ui.label(egui::RichText::new(key).monospace().weak().small());
                }
                ui.add_space(4.0);
            });

        ui.label(description);
    });
}
