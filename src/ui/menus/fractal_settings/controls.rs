use crate::ui::window::WindowParams;

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
        _ctx: &mut crate::ui::window::WindowContext,
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

            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);

                ui.button("[PH] screenshot");
                ui.button("[PH] save paramaters");
                ui.button("[PH] load from file");

                ui.add_space(50.0);

                if ui.button("[PH] Save").clicked() {
                    close = true;
                }
            });
        });

        close
    }
}
