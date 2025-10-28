use crate::ui::window::{Window, WindowParams};

pub struct Controls {
    pub params: WindowParams,
    is_open: bool,
}
impl Controls {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            is_open: true,
        }
    }
}
impl Window for Controls {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }

    fn update(&mut self, egui_ctx: &egui::Context, _ctx: &mut crate::ui::window::WindowContext) {
        self.params.sized_area("controls", egui_ctx, |ui| {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().left(), ui.max_rect().bottom()),
                    egui::Pos2::new(ui.max_rect().right(), ui.max_rect().bottom()),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLACK),
            );
        });
    }
}
