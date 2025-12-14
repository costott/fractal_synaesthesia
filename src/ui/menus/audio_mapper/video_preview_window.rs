use std::sync::Arc;

use egui::Widget;

use crate::{
    audio::{analyzer::SongFeatures, song::Song},
    rendering::{
        algorithms::render_algorithms::FractalParams,
        fractal_visualiser::FractalVisualiser,
        video::{
            VideoFrameManager, VideoManager,
            audio_mapper::{self, AudioMapper},
        },
    },
    ui::{
        fractal::fractal_canvas::CanvasDimensions,
        menus::audio_mapper::{AudioMapperWindow, audio_player::AudioPlayer},
        window::WindowParams,
    },
};

pub struct VideoPreviewWindow {
    params: WindowParams,

    preview_visualiser: FractalVisualiser,
    song_preview_playback: Option<AudioPlayer>,

    rendered_timestamp: Option<f64>,
}
impl VideoPreviewWindow {
    pub fn new(
        params: WindowParams,
        ctx: &super::AudioMapperContext,
        canvas_dims: CanvasDimensions,
        initial_fractal_params: &FractalParams,
    ) -> Self {
        let mut preview_visualiser = FractalVisualiser::new(
            initial_fractal_params,
            canvas_dims,
            Arc::clone(&ctx.layer_manager),
            1,
            true,
        );

        preview_visualiser.update_render(Arc::new(std::sync::Mutex::new(
            initial_fractal_params.clone(),
        )));

        Self {
            params,
            preview_visualiser,
            song_preview_playback: None,
            rendered_timestamp: None,
        }
    }

    fn load_song(&mut self, song_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let audio_player = AudioPlayer::new(song_path)?
            .set_slider_width(self.preview_visualiser.canvas.dims.width as f32 * 0.75);

        self.song_preview_playback = Some(audio_player);

        Ok(())
    }

    pub fn updated_context(&mut self, ctx: &super::AudioMapperContext) {
        if let Some(song_path) = &ctx.song_path {
            let _ = self.load_song(song_path);
        }
    }

    pub fn change_dimensions(&mut self, canvas_dims: CanvasDimensions) {
        self.preview_visualiser.change_dimensions(canvas_dims);
    }

    pub fn draw(&self) {
        self.preview_visualiser.draw(
            self.params.x as f32 + self.params.width as f32 * 0.5
                - self.preview_visualiser.canvas.dims.width as f32 * 0.5,
            self.params.y as f32 + 20.0,
        );
    }

    pub fn update(
        &mut self,
        _egui_ctx: &egui::Context,
        ctx: &mut super::AudioMapperContext,
        video_manager: &mut VideoManager,
        audio_mapper: &AudioMapper,
    ) {
        if let Some(audio_player) = self.song_preview_playback.as_ref() {
            let timestamp = audio_player.get_timestamp();

            // Avoid re-rendering the same timestamp multiple times
            let mut started_timestamp = false;
            if let Some(rendered_timestamp) = self.rendered_timestamp {
                if (rendered_timestamp - timestamp).abs() < 0.1 {
                    video_manager.try_end_render_frame(&mut self.preview_visualiser);
                    started_timestamp = true;
                }
            }

            if !started_timestamp {
                self.rendered_timestamp = Some(timestamp);

                let video_percent =
                    timestamp as f32 / ctx.song_manager.as_ref().unwrap().song.duration();
                video_manager.start_render_frame(
                    &mut self.preview_visualiser,
                    audio_mapper,
                    ctx.song_manager.as_ref().unwrap(),
                    video_percent,
                );
            }
        }

        self.preview_visualiser.improve_quality();

        self.params
            .sized_area("video_preview_window", _egui_ctx, |ui| {
                ui.painter().rect_stroke(
                    ui.max_rect(),
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                    egui::StrokeKind::Middle,
                );

                ui.add_space(20.0 + self.preview_visualiser.canvas.dims.height as f32);

                ui.vertical_centered(|ui| {
                    if let Some(audio_player) = self.song_preview_playback.as_mut() {
                        audio_player.ui(ui);
                    }
                });
            });
    }
}
