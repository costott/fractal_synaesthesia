use crate::ui::window::WindowParams;

pub struct VideoSettings {
    pub params: WindowParams,

    lock_aspect_ratio: bool,
}
impl VideoSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            lock_aspect_ratio: true,
        }
    }

    /// Returns true if the video aspect ratio was changed.
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut super::AudioMapperContext,
    ) -> bool {
        let mut changed_dims = false;

        self.params.sized_area("Video Settings", egui_ctx, |ui| {
            egui::Frame::new()
                .fill(super::BACKGROUND_SECONDARY_COLOUR)
                .outer_margin(egui::Margin::symmetric(30, 110))
                .inner_margin(egui::vec2(15.0, 15.0))
                .corner_radius(10.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Video Settings");
                        ui.separator();

                        ui.horizontal(|ui| {
                            egui::Grid::new("parameters_grid")
                                .num_columns(2)
                                .show(ui, |ui| {
                                    let original_dims = ctx.video_dimensions;

                                    ui.label("Width");
                                    let response = ui.add(
                                        egui::DragValue::new(&mut ctx.video_dimensions.width)
                                            .speed(1)
                                            .range(1..=50000),
                                    );
                                    if response.changed() {
                                        if self.lock_aspect_ratio {
                                            let new_dims = original_dims
                                                .new_from_this_aspect_with_width(
                                                    ctx.video_dimensions.width,
                                                );
                                            ctx.video_dimensions.height = new_dims.height;

                                            // Prevent zero height.
                                            if ctx.video_dimensions.height < 100 {
                                                ctx.video_dimensions = original_dims
                                            }
                                        } else {
                                            changed_dims = true;
                                        }
                                    }

                                    ui.end_row();

                                    ui.label("Height");
                                    let response = ui.add(
                                        egui::DragValue::new(&mut ctx.video_dimensions.height)
                                            .speed(1)
                                            .range(1..=50000),
                                    );
                                    if response.changed() {
                                        if self.lock_aspect_ratio {
                                            let new_dims = original_dims
                                                .new_from_this_aspect_with_height(
                                                    ctx.video_dimensions.height,
                                                );
                                            ctx.video_dimensions.width = new_dims.width;

                                            // Prevent zero width.
                                            if ctx.video_dimensions.width < 100 {
                                                ctx.video_dimensions = original_dims
                                            }
                                        } else {
                                            changed_dims = true;
                                        }
                                    }
                                });

                            ui.checkbox(&mut self.lock_aspect_ratio, "Lock Aspect Ratio");
                        });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("FPS");
                            let mut fps_copy = project.fps();
                            ui.add(egui::DragValue::new(&mut fps_copy).speed(1).range(1..=240));
                            if fps_copy != project.fps() {
                                ctx.change_fps(project, fps_copy);
                            }
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            let total_frames = if let Some(song_manager) = &ctx.song_manager {
                                &format!("{}", song_manager.total_frames_at_fps(project.fps()))
                            } else {
                                "N/A"
                            };
                            ui.label(format!("Total frames: {}", total_frames));
                        });
                    });
                });
        });

        changed_dims
    }
}
