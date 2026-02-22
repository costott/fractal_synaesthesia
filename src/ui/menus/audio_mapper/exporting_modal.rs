use macroquad::prelude::*;

use crate::{
    exporting::Exporter, rendering::algorithms::render_algorithms::FractalParams,
    ui::menus::audio_mapper::AudioMapperWindow,
};

pub struct ExportingModal {
    exporter: Exporter,

    progress_bar: ExportingProgressBar,
}
impl ExportingModal {
    pub fn new(
        end_params: &FractalParams,
        project: &crate::project::Project,
        ctx: &super::AudioMapperContext,
    ) -> Self {
        Self {
            exporter: Exporter::new(0, project, ctx, end_params).unwrap(),
            progress_bar: ExportingProgressBar::start(0.0),
        }
    }
}
impl AudioMapperWindow for ExportingModal {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut super::AudioMapperContext,
    ) {
        self.exporter.update(project, ctx).unwrap();
        self.progress_bar.update(self.exporter.get_progress(ctx));

        egui::Modal::new("export_popup".into()).show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Exporting...");
                ui.separator();
                ui.label("Your video is currently being exported. Please wait...");

                match self.exporter.state {
                    crate::exporting::ExporterState::RenderingFrames => {
                        self.progress_bar.show(ui);
                    }
                    crate::exporting::ExporterState::EncodingVideo => {
                        ui.label("Encoding video...");
                    }
                    crate::exporting::ExporterState::MuxingAudio => {
                        ui.label("Muxing audio...");
                    }
                    crate::exporting::ExporterState::Finished => {
                        ui.label("Export finished!");
                    }
                }
            });
        });
    }
}

struct ExportingProgressBar {
    start_time: std::time::Instant,
    start_progress: f32,

    saved_current_progress: f32,
}
impl ExportingProgressBar {
    pub fn start(start_progress: f32) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            start_progress,
            saved_current_progress: start_progress,
        }
    }

    pub fn update(&mut self, current_progress: f32) {
        self.saved_current_progress = current_progress;
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress_delta = self.saved_current_progress - self.start_progress;
        let estimated_total_time = if progress_delta > 0.0 {
            elapsed / progress_delta
        } else {
            0.0
        };
        let estimated_time_left = estimated_total_time * (1.0 - self.saved_current_progress);

        let hms = {
            let total_seconds = estimated_time_left as u64;
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        };
        ui.label(format!(
            "Progress: {:.2}%, Estimated time left: {}",
            self.saved_current_progress * 100.0,
            hms
        ));
        ui.add(
            egui::ProgressBar::new(self.saved_current_progress)
                .show_percentage()
                .animate(true),
        );
    }
}
