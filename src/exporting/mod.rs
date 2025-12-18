pub mod video_exporter_ffmpeg;
pub use video_exporter_ffmpeg::VideoExporterFFmpeg;

use macroquad::prelude::*;
use std::{sync::Arc, thread, time::Duration};

use crate::rendering::fractal_visualiser::FractalVisualiser;

/// Simple Exporter that orchestrates frame rendering and writes frames via FFmpeg exporter
pub struct Exporter {
    current_frame: usize,
    rendering_frame: bool,

    visualiser: FractalVisualiser,

    pub finished: bool,
    ffmpeg_exporter: VideoExporterFFmpeg,
}
impl Exporter {
    pub fn new(
        start_frame: usize,
        ctx: &crate::ui::menus::audio_mapper::AudioMapperContext,
        end_params: &crate::rendering::algorithms::render_algorithms::FractalParams,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            current_frame: start_frame,
            rendering_frame: false,
            visualiser: FractalVisualiser::new(
                end_params,
                ctx.video_dimensions,
                Arc::clone(&ctx.layer_manager),
                1,
                false,
            ),
            finished: false,
            ffmpeg_exporter: VideoExporterFFmpeg::new(
                ctx.destination_file_path
                    .as_ref()
                    .expect("no destination file path set"),
                ctx.video_dimensions.width as u32,
                ctx.video_dimensions.height as u32,
                ctx.fps as u32,
            )?,
        })
    }

    pub fn update(
        &mut self,
        ctx: &mut crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_frame >= ctx.video_manager.total_frames {
            if !self.finished {
                self.finish(ctx)?;
            }
            return Ok(());
        }

        if self.rendering_frame {
            if self.visualiser.finished_render() {
                self.rendering_frame = false;
                self.save_current_frame(ctx)?;
            } else {
                thread::sleep(Duration::from_millis(10));
                return Ok(());
            }
        }

        self.start_current_frame(ctx)?;

        Ok(())
    }

    fn start_current_frame(
        &mut self,
        ctx: &mut crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let song_manager = ctx
            .song_manager
            .as_ref()
            .ok_or("no song loaded in AudioMapperContext")?;

        let duration = song_manager.song.duration();
        let total_frames = (duration * ctx.fps as f32) as usize;

        if self.current_frame >= total_frames {
            return Ok(());
        }

        let video_percent = self.current_frame as f32 / total_frames as f32;

        ctx.video_manager.start_render_frame(
            &mut self.visualiser,
            &ctx.audio_mapper,
            song_manager,
            video_percent,
        );

        self.rendering_frame = true;

        Ok(())
    }

    fn save_current_frame(
        &mut self,
        ctx: &mut crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // grab the rendered image (macroquad Image stores RGBA8 bytes)
        let image_arc = self.visualiser.rendered_image();
        let img = image_arc.lock().unwrap();

        // Store image as a png in intermediate temp folder
        if ctx.intermediate_pngs {
            let intermidate_dir = ctx
                .destination_file_path
                .as_ref()
                .map(|p| {
                    let mut path = p.clone();
                    path.set_extension("");
                    path
                })
                .expect("no destination file path set");

            std::fs::create_dir_all(&intermidate_dir)?;
            let frame_path = intermidate_dir.join(format!("frame_{:05}.png", self.current_frame));
            img.export_png(frame_path.to_str().expect("invalid frame path"));
        } else {
            // Instead, push frame directly to ffmpeg exporter
            let rgb_data = self.image_to_rgb24(&img);
            self.ffmpeg_exporter.push_frame(&rgb_data)?;
        }

        ctx.video_manager
            .force_end_render_frame(&mut self.visualiser);
        self.current_frame += 1;

        Ok(())
    }

    fn image_to_rgb24(&self, img: &Image) -> Vec<u8> {
        let rgba = &img.bytes;
        let width = img.width as usize;
        let height = img.height as usize;

        let mut rgb = Vec::with_capacity(width * height * 3);
        for px in 0..(width * height) {
            let base = px * 4;
            rgb.push(rgba[base]);
            rgb.push(rgba[base + 1]);
            rgb.push(rgba[base + 2]);
        }

        rgb
    }

    fn finish(
        &mut self,
        ctx: &crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if ctx.intermediate_pngs {
            // Gather all intermediate pngs and push to ffmpeg exporter
            let intermidate_dir = ctx
                .destination_file_path
                .as_ref()
                .map(|p| {
                    let mut path = p.clone();
                    path.set_extension("");
                    path
                })
                .expect("no destination file path set");

            for frame_idx in 0..ctx.video_manager.total_frames {
                let frame_path = intermidate_dir.join(format!("frame_{:05}.png", frame_idx));
                let img = futures::executor::block_on(async {
                    macroquad::prelude::load_image(
                        &frame_path.to_str().expect("invalid frame path"),
                    )
                    .await
                })?;
                let rgb_data = self.image_to_rgb24(&img);
                self.ffmpeg_exporter.push_frame(&rgb_data)?;
            }

            self.ffmpeg_exporter.finish()?;

            // Clean up intermediate pngs
            std::fs::remove_dir_all(&intermidate_dir)?;
        }

        // TODO: add the audio track to the mp4

        self.finished = true;
        Ok(())
    }

    pub fn get_progress(&self, ctx: &crate::ui::menus::audio_mapper::AudioMapperContext) -> f32 {
        self.current_frame as f32 / ctx.video_manager.total_frames as f32
    }
}
