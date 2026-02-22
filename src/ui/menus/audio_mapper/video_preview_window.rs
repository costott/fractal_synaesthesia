use std::sync::{Arc, Mutex};

use egui::Widget;

use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams, fractal_visualiser::FractalVisualiser,
    },
    ui::{
        fractal::fractal_canvas::CanvasDimensions, menus::audio_mapper::audio_player::AudioPlayer,
        window::WindowParams,
    },
};

pub struct VideoPreviewWindow {
    params: WindowParams,

    fixed_preview_height: u16,

    preview_visualiser: FractalVisualiser,
    song_preview_playback: Option<AudioPlayer>,

    rendered_timestamp: Option<f64>,
    are_rendering: bool,
}
impl VideoPreviewWindow {
    pub fn new(
        params: WindowParams,
        project: &crate::project::Project,
        unscaled_canvas_dims: CanvasDimensions,
        fixed_preview_height: u16,
        initial_fractal_params: Arc<Mutex<FractalParams>>,
    ) -> Self {
        let mut preview_visualiser = FractalVisualiser::new(
            initial_fractal_params.clone(),
            unscaled_canvas_dims.new_from_this_aspect_with_height(fixed_preview_height),
            Arc::clone(&project.fractal_settings.layers),
            1,
            true,
        );

        preview_visualiser.update_render(initial_fractal_params.clone());

        Self {
            params,
            fixed_preview_height,
            preview_visualiser,
            song_preview_playback: None,
            rendered_timestamp: None,
            are_rendering: false,
        }
    }

    pub fn changed_fractal_settings(&mut self, project: &crate::project::Project) {
        self.preview_visualiser
            .set_layer_manager(project.fractal_settings.layers.clone());
        self.preview_visualiser
            .update_render(project.fractal_settings.params.clone());
    }

    fn load_song(&mut self, song_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let audio_player = AudioPlayer::new(song_path)?
            .set_slider_width(self.preview_visualiser.canvas.dims.width as f32 * 0.75);

        self.song_preview_playback = Some(audio_player);

        Ok(())
    }

    pub fn updated_song(&mut self, project: &crate::project::Project) {
        if let Some(song_path) = &project.song_path {
            let _ = self.load_song(song_path);
        }
    }

    pub fn change_dimensions(&mut self, canvas_dims: CanvasDimensions) {
        self.preview_visualiser.change_dimensions(
            canvas_dims.new_from_this_aspect_with_height(self.fixed_preview_height),
        );
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
        project: &crate::project::Project,
        ctx: &mut super::AudioMapperContext,
    ) {
        if ctx.exporting {
            return;
        }

        if let Some(audio_player) = self.song_preview_playback.as_ref() {
            let timestamp = audio_player.get_timestamp();

            // Avoid re-rendering the same timestamp multiple times
            let mut started_timestamp = false;
            if let Some(rendered_timestamp) = self.rendered_timestamp {
                if (rendered_timestamp - timestamp).abs() < 0.1 {
                    if let Some(_) = ctx
                        .video_manager
                        .try_end_render_frame(&mut self.preview_visualiser)
                    {
                        self.are_rendering = false;
                    }
                    started_timestamp = true;
                }
            }

            if !started_timestamp && self.are_rendering {
                ctx.video_manager
                    .force_end_render_frame(&mut self.preview_visualiser);
                self.are_rendering = false;
            }

            if !started_timestamp {
                self.rendered_timestamp = Some(timestamp);

                let video_percent =
                    timestamp as f32 / ctx.song_manager.as_ref().unwrap().song.duration();
                ctx.video_manager.start_render_frame(
                    &mut self.preview_visualiser,
                    &project.audio_mapper,
                    ctx.song_manager.as_ref().unwrap(),
                    video_percent,
                );
                self.are_rendering = true;
            }
        }

        self.preview_visualiser.improve_quality();

        self.params
            .sized_area("video_preview_window", _egui_ctx, |ui| {
                ui.add_space(20.0 + self.preview_visualiser.canvas.dims.height as f32);

                ui.vertical_centered(|ui| {
                    if let Some(audio_player) = self.song_preview_playback.as_mut() {
                        audio_player.ui(ui);
                    }
                });
            });
    }
}
