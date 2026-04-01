use crate::ui::window::WindowParams;

pub struct AudioMapperFooter {
    params: WindowParams,
}
impl AudioMapperFooter {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }

    /// Returns true to switch back to fractal settings
    pub fn update(&mut self, egui_ctx: &egui::Context) -> bool {
        let mut return_to_fractal_settings = false;

        self.params
            .sized_area("Audio Mapper Footer", egui_ctx, |ui| {
                ui.painter().line_segment(
                    [
                        egui::pos2(self.params.x as f32, self.params.y as f32),
                        egui::pos2(
                            (self.params.x + self.params.width) as f32,
                            self.params.y as f32,
                        ),
                    ],
                    (1.0, egui::Color32::GRAY),
                );

                ui.horizontal_centered(|ui| {
                    ui.add_space(5.0);
                    if ui.button("Edit fractal").clicked() {
                        return_to_fractal_settings = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(5.0);
                        ui.label("Fractal Synaesthesia");
                    });
                });
            });

        return_to_fractal_settings
    }
}
