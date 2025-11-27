use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct WindowParams {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
}
impl WindowParams {
    pub fn get_bounding_rect(&self) -> Rect {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }

    pub fn sized_area(
        &self,
        source: impl std::hash::Hash,
        egui_ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        egui::Area::new(egui::Id::new(source))
            .fixed_pos(egui::pos2(self.x as f32, self.y as f32))
            .show(egui_ctx, |ui| {
                ui.set_min_size(egui::vec2(self.width as f32, self.height as f32));
                ui.set_max_size(egui::vec2(self.width as f32, self.height as f32));

                add_contents(ui);
            });
    }
}
