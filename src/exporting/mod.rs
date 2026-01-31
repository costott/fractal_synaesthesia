pub mod video_exporter;
pub use video_exporter::VideoExporter;
mod audio_muxer;
mod audio_muxer_ffmpeg;
mod extract_ffmpeg;
mod video_exporter_ffmpeg;

use macroquad::prelude::*;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::rendering::fractal_visualiser::FractalVisualiser;

pub enum ExporterState {
    RenderingFrames,
    EncodingVideo,
    MuxingAudio,
    Finished,
}

/// Simple Exporter that orchestrates frame rendering and writes frames via FFmpeg exporter
pub struct Exporter {
    pub state: ExporterState,

    current_frame: usize,
    rendering_frame: bool,

    visualiser: FractalVisualiser,

    pub finished: bool,
    // ffmpeg_exporter: VideoExporter,
    ffmpeg_exporter: video_exporter_ffmpeg::FFmpegCmdExporter,
    video_only_path: std::path::PathBuf,
}
impl Exporter {
    pub fn new(
        start_frame: usize,
        project: &crate::project::Project,
        ctx: &crate::ui::menus::audio_mapper::AudioMapperContext,
        end_params: &crate::rendering::algorithms::render_algorithms::FractalParams,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // change e.g. "output.mp4" to "output_video_only.mp4" for ffmpeg exporter
        let mut video_only_path = ctx
            .destination_file_path
            .as_ref()
            .expect("no destination file path set")
            .clone();
        let stem = video_only_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid destination file path")?;
        let parent = video_only_path
            .parent()
            .ok_or("invalid destination file path")?;
        video_only_path = parent.join(format!("{}_video_only.mp4", stem));

        let owned_layer_manager = project.fractal_settings.layers.lock().unwrap().clone();

        Ok(Self {
            state: ExporterState::RenderingFrames,
            current_frame: start_frame,
            rendering_frame: false,
            visualiser: FractalVisualiser::new(
                std::sync::Arc::new(std::sync::Mutex::new(end_params.clone())),
                ctx.video_dimensions,
                Arc::new(Mutex::new(owned_layer_manager)),
                1,
                false,
            ),
            finished: false,
            ffmpeg_exporter: video_exporter_ffmpeg::FFmpegCmdExporter::new(
                ctx.video_dimensions.width as u32,
                ctx.video_dimensions.height as u32,
                project.fps() as u32,
                &video_only_path,
            )?,
            video_only_path,
        })
    }

    pub fn update(
        &mut self,
        project: &crate::project::Project,
        ctx: &mut crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_frame >= ctx.video_manager.total_frames {
            if !self.finished {
                self.finish(project, ctx)?;
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

        self.start_current_frame(project, ctx)?;

        Ok(())
    }

    fn start_current_frame(
        &mut self,
        project: &crate::project::Project,
        ctx: &mut crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let song_manager = ctx
            .song_manager
            .as_ref()
            .ok_or("no song loaded in AudioMapperContext")?;

        let duration = song_manager.song.duration();
        let total_frames = (duration * project.fps() as f32) as usize;

        if self.current_frame >= total_frames {
            return Ok(());
        }

        let video_percent = self.current_frame as f32 / total_frames as f32;

        ctx.video_manager.start_render_frame(
            &mut self.visualiser,
            &project.audio_mapper,
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
            .force_end_render_frame(&mut self.visualiser)
            .expect("something went wrong getting ready for next frame");
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
        project: &crate::project::Project,
        ctx: &crate::ui::menus::audio_mapper::AudioMapperContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.state = ExporterState::EncodingVideo;

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

            // Clean up intermediate pngs
            std::fs::remove_dir_all(&intermidate_dir)?;
        }

        self.ffmpeg_exporter.finish()?;

        self.state = ExporterState::MuxingAudio;

        audio_muxer_ffmpeg::mux_audio_video(
            &self.video_only_path,
            &project.song_path.as_ref().unwrap().into(),
            &ctx.destination_file_path.as_ref().unwrap().clone(),
        )?;

        self.state = ExporterState::Finished;

        std::fs::remove_file(&self.video_only_path)?;

        self.finished = true;
        Ok(())
    }

    pub fn get_progress(&self, ctx: &crate::ui::menus::audio_mapper::AudioMapperContext) -> f32 {
        self.current_frame as f32 / ctx.video_manager.total_frames as f32
    }
}
