use crate::ui::window::{WindowParams, close_app, minimize_app};

pub struct Controls {
    pub params: WindowParams,
}
impl Controls {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }

    /// Returns whether or not the fractal settings mode should be saved and closed
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        _ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) -> bool {
        let mut close = false;

        self.params.sized_area("controls", egui_ctx, |ui| {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().left(), ui.max_rect().bottom()),
                    egui::Pos2::new(ui.max_rect().right(), ui.max_rect().bottom()),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLACK),
            );

            // change background colour
            let background_colour = egui::Color32::from_gray(255);
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, background_colour);

            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);

                ui.heading("Edit Fractal Settings");

                ui.style_mut().spacing.button_padding = egui::vec2(10.0, 10.0);
                if ui.button("Done").clicked() {
                    close = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // helper to draw an icon button with custom hover/bg behaviour
                    let draw_icon_button = |ui: &mut egui::Ui,
                                            icon: &str,
                                            hover_color: Option<egui::Color32>|
                     -> egui::Response {
                        let size = egui::vec2(
                            (self.params.height - 2) as f32,
                            (self.params.height - 2) as f32,
                        );
                        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());

                        // base background - match normal widget background
                        let base_bg = background_colour;

                        // choose fill depending on hover state
                        let fill = if resp.hovered() {
                            hover_color.unwrap_or_else(|| {
                                // darken base for default hover (used for minimise)
                                let r = (base_bg.r() as f32 * 0.8) as u8;
                                let g = (base_bg.g() as f32 * 0.8) as u8;
                                let b = (base_bg.b() as f32 * 0.8) as u8;
                                egui::Color32::from_rgb(r, g, b)
                            })
                        } else {
                            base_bg
                        };

                        ui.painter().rect_filled(rect, 0.0, fill);

                        // text color - use visuals text color for contrast
                        let text_color = ui.visuals().text_color();
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon,
                            egui::FontId::proportional((self.params.height as f32) * 0.5),
                            text_color,
                        );

                        resp
                    };

                    // Close button
                    if draw_icon_button(ui, "✖", Some(egui::Color32::from_rgb(0xFF, 0x44, 0x44)))
                        .clicked()
                    {
                        close_app();
                    }

                    // ensure no space between the buttons
                    ui.add_space(-ui.spacing().item_spacing.x);

                    // Minimise button
                    if draw_icon_button(ui, "–", None).clicked() {
                        minimize_app();
                    }
                });
            });
        });

        close
    }
}
