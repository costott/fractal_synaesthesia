use crate::ui::window::WindowParams;

pub struct ExportMenu {
    pub params: WindowParams,
}
impl ExportMenu {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context, ctx: &mut super::AudioMapperContext) {
        self.params.sized_area("Export menu", egui_ctx, |ui| {
            egui::Frame::new()
                .fill(super::BACKGROUND_SECONDARY_COLOUR)
                .outer_margin(egui::Margin::symmetric(30, 100))
                .inner_margin(egui::vec2(15.0, 15.0))
                .corner_radius(10.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Export Video");
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Destination:");
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}",
                                    ctx.destination_file_path.as_ref().map_or("Not set", |p| p
                                        .to_str()
                                        .unwrap_or("Invalid path"))
                                ))
                                .weak(),
                            );

                            if ui.button("🗁").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Select Destination File")
                                    .set_file_name("output.mp4")
                                    .add_filter("MP4 Video", &["mp4"])
                                    .set_directory(ctx.destination_file_path.as_ref().and_then(|p| p.parent()).unwrap_or_else(|| ".".as_ref()))
                                    .save_file()
                                {
                                    ctx.destination_file_path = Some(path);
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.checkbox(
                                &mut ctx.intermediate_pngs, 
                                "Save intermediate PNGs (RECOMMENDED)"
                            ).on_hover_text("Save each frame as a PNG file in a temporary folder during export. This can help with stability and allows for easier recovery if the export is interrupted.");
                        });

                        ui.horizontal(|ui| {
                            if ctx.intermediate_pngs {
                                let path_no_mp4 = ctx.destination_file_path.as_ref().map(|p| {
                                    let mut path = p.clone();
                                    path.set_extension("");
                                    path
                                });
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Intermediate PNGs will be saved to: {}",
                                        path_no_mp4.as_ref().map_or("Desination not set", |p| p
                                            .to_str()
                                            .unwrap_or("Invalid path"))
                                    ))
                                    .weak(),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("Warning: If export interruped, all progress will be lost.")
                                        .weak()
                                        .color(egui::Color32::ORANGE),
                                );
                            }
                        });

                        ui.separator();

                        if ctx.song_manager.is_some() {
                            if ui.add(egui::Button::new("Export").min_size(egui::Vec2::new(100., 60.))).clicked() {
                                ctx.exporting = true;
                            }
                        }
                    });
                });
        });
    }
}
