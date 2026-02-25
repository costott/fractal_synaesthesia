use crate::ui::{menus::audio_mapper::AudioMapperWindow, window::WindowParams};

pub struct ProjectManager {
    params: WindowParams,
}
impl ProjectManager {
    pub fn new(params: WindowParams) -> Self {
        Self { params }
    }
}
impl AudioMapperWindow for ProjectManager {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        _ctx: &mut super::AudioMapperContext,
    ) {
        egui::Area::new(egui::Id::new("project manager"))
            .fixed_pos(egui::pos2(self.params.x as f32, self.params.y as f32))
            .order(egui::Order::Foreground)
            .show(egui_ctx, |ui| {
                ui.set_min_size(egui::vec2(
                    self.params.width as f32,
                    self.params.height as f32,
                ));
                ui.set_max_size(egui::vec2(
                    self.params.width as f32,
                    self.params.height as f32,
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Load Project").clicked() {
                        // load project
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Project File")
                            .add_filter("Fractal Synaesthesia Project", &["fsp"])
                            .set_directory(
                                dirs::document_dir()
                                    .as_ref()
                                    .and_then(|p| Some(p.as_path()))
                                    .unwrap_or_else(|| ".".as_ref()),
                            )
                            .pick_file()
                        {
                            if let Ok(json) = std::fs::read_to_string(path) {
                                if let Ok(loaded_project) =
                                    serde_json::from_str::<crate::project::Project>(&json)
                                {
                                    *project = loaded_project;
                                }
                            }
                        }
                    }
                    if ui.button("Save Project").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Destination File")
                            .set_file_name("video_project.fsp")
                            .add_filter("Fractal Synaesthesia Project", &["fsp"])
                            .set_directory(
                                dirs::document_dir()
                                    .as_ref()
                                    .and_then(|p| Some(p.as_path()))
                                    .unwrap_or_else(|| ".".as_ref()),
                            )
                            .save_file()
                        {
                            // save project to path using serde_json
                            let json = serde_json::to_string_pretty(&project).unwrap();
                            std::fs::write(path, json).unwrap_or(());
                        }
                    }
                });
            });
    }
}
