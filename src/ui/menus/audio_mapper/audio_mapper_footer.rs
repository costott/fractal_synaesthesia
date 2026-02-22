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
                    if ui.button("Edit fractal").clicked() {
                        return_to_fractal_settings = true;
                    }
                });
            });

        return_to_fractal_settings
    }
}
