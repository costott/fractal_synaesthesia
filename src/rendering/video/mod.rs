use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{Fractal, FractalParams},
        fractal_visualiser::FractalVisualiser,
        manager::layer_manager::LayerManager,
        video::{audio_mapper::AudioMapper, song_manager::SongManager},
    },
    types::BigComplex,
    ui::{FractalSettings, menus::audio_mapper::AudioMapperContext},
};

pub mod audio_mapper;
pub mod song_manager;

pub struct VideoManager {
    frame_manager: VideoFrameManager,
    zoom_timeline: ZoomTimeline,
    rotation_timeline: RotationTimeline,

    total_frames: usize,
    framerate: f32,
    hop_size: f32,

    /// Stored to restore after rendering
    original_layers: Option<Arc<Mutex<LayerManager>>>,
    start_rotation: f64,
}
impl VideoManager {
    /// Creates a new video manager for rendering videos based on the provided song manager
    /// and fractal parameters
    ///
    /// # Params
    /// - `context`: Audio mapper context
    /// - `params`: Fractal parameters of end frame
    /// - `framerate`: Framerate of the video to be rendered
    /// - `hop_size`: Hop size used for the granularity of zoom timeline sampling
    pub fn new(
        context: &AudioMapperContext,
        params: &FractalParams,
        framerate: f32,
        hop_size: f32,
    ) -> Self {
        let duration = context
            .song_manager
            .as_ref()
            .map(|sm| sm.song.duration())
            .unwrap_or(60.0);
        let total_frames = (duration * framerate) as usize;

        Self {
            zoom_timeline: ZoomTimeline::linear(
                crate::ui::menus::fractal_settings::START_PIXEL_STEP,
                params.pixel_step,
                duration,
                hop_size,
            ),
            rotation_timeline: RotationTimeline::empty(params.rotation, duration, hop_size),
            frame_manager: VideoFrameManager::new(params.clone()),
            total_frames,
            framerate,
            hop_size,
            original_layers: None,
            start_rotation: params.rotation,
        }
    }

    pub fn updated_context(&mut self, context: &AudioMapperContext) {
        self.total_frames =
            (context.song_manager.as_ref().unwrap().song.duration() * self.framerate) as usize;
    }

    /// Rebuild the zoom+rotation timeline based on the updated audio mapper and context
    pub fn updated_audio_mapper(&mut self, context: &AudioMapperContext) -> Option<()> {
        self.zoom_timeline = ZoomTimeline::build(
            context.song_manager.as_ref()?,
            context.layer_manager.clone(),
            &context.audio_mapper,
            crate::ui::menus::fractal_settings::START_PIXEL_STEP,
            self.zoom_timeline.final_pixel_step,
            self.hop_size,
        );
        self.rotation_timeline = RotationTimeline::build(
            context.song_manager.as_ref()?,
            context.layer_manager.clone(),
            &context.audio_mapper,
            self.start_rotation,
            self.hop_size,
        );
        Some(())
    }

    pub fn start_render_frame(
        &mut self,
        visualiser: &mut FractalVisualiser,
        audio_mapper: &AudioMapper,
        song_manager: &SongManager,
        video_percent: f32,
    ) {
        let timestamp = (self.total_frames as f32 * video_percent) / self.framerate;

        let current_magnification = crate::ui::menus::fractal_settings::START_PIXEL_STEP
            / self.zoom_timeline.sample_at(timestamp);
        let frame = self.frame_manager.get_frame(10.0, current_magnification);

        let render_layers = song_manager.get_layers_at_timestamp(
            visualiser.layer_manager.clone(),
            audio_mapper,
            timestamp,
        );

        let frame_params = Arc::new(Mutex::new(FractalParams::new(
            Fractal::Mandelbrot { power: 2 },
            frame.center,
            self.zoom_timeline.sample_at(timestamp),
            frame.max_iterations,
            self.rotation_timeline.sample_at(timestamp),
        )));

        self.original_layers = Some(visualiser.layer_manager.clone());
        visualiser.layer_manager = Arc::new(Mutex::new(render_layers));

        visualiser.update_render(frame_params);
    }

    pub fn try_end_render_frame(&mut self, visualiser: &mut FractalVisualiser) -> Option<()> {
        if !visualiser.finished_render() {
            return None;
        }

        visualiser.layer_manager = self.original_layers.take()?;
        Some(())
    }

    pub fn force_end_render_frame(&mut self, visualiser: &mut FractalVisualiser) -> Option<()> {
        visualiser.layer_manager = self.original_layers.take()?;
        Some(())
    }
}

/// Timeline of zoom levels over time
///
/// This needs to be built ahead of time to allow for proper interpolation, as the zoom
/// levels depend on audio features over the entire song.
struct ZoomTimeline {
    /// (timestamp, pixel_step)
    samples: Vec<(f32, f64)>,
    final_pixel_step: f64,
}
impl ZoomTimeline {
    /// Creates a linear zoom timeline from start to end pixel step over the given duration
    pub fn linear(
        start_pixel_step: f64,
        end_pixel_step: f64,
        duration: f32,
        hop_size: f32,
    ) -> Self {
        let mut samples = Vec::new();

        let step_change_per_second = (end_pixel_step - start_pixel_step) / duration as f64;

        let mut t = 0.0;
        while t < duration {
            let w = start_pixel_step + step_change_per_second * t as f64;
            samples.push((t, w));
            t += hop_size;
        }

        Self {
            samples,
            final_pixel_step: end_pixel_step,
        }
    }

    pub fn build(
        song_manager: &SongManager,
        layer_manager: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        start_pixel_step: f64,
        final_pixel_step: f64,
        hop_size: f32,
    ) -> Self {
        let samples = (song_manager.song.duration() / hop_size) as usize;

        // integrate zoom multipliers over time
        let mut cumulative = Vec::with_capacity(samples);
        let mut t = 0.0;
        let mut sum = 0.0; // total weighted zoom amount W
        for _ in 0..samples {
            let m = song_manager.get_zoom_multiplier_at_timestamp(
                layer_manager.clone(),
                audio_mapper,
                t,
            );
            sum += m;
            cumulative.push((t, sum));
            t += hop_size;
        }

        // normalise cumulative multipliers to span from start_pixel_step to final_pixel_step
        let log_p0 = start_pixel_step.ln();
        let log_p1 = final_pixel_step.ln();
        for (_, m) in cumulative.iter_mut() {
            let u = *m / sum;
            let log_p = (1.0 - u) * log_p0 + u * log_p1;
            *m = log_p.exp();
        }

        Self {
            samples: cumulative,
            final_pixel_step,
        }
    }

    pub fn sample_at(&self, timestamp: f32) -> f64 {
        if timestamp <= self.samples[0].0 {
            return self.samples[0].1;
        }
        if timestamp >= self.samples[self.samples.len() - 1].0 {
            return self.samples[self.samples.len() - 1].1;
        }

        let idx = match self
            .samples
            .binary_search_by(|(time, _)| time.partial_cmp(&timestamp).unwrap())
        {
            Ok(i) => return self.samples[i].1, // exact hit
            Err(i) => i,
        };

        let (t1, v1) = self.samples[idx - 1];
        let (t2, v2) = self.samples[idx];

        let alpha = (timestamp - t1) / (t2 - t1);
        v1 + (v2 - v1) * alpha as f64
    }
}

/// Timeline of rotation over time
///
/// This needs to be built ahead of time as rotation is cumulative,
/// but we need a deterministic way to get the rotation at any timestamp without
/// knowing the previous timestamp's rotation.
struct RotationTimeline {
    /// (timestamp, rotation in radians)
    samples: Vec<(f32, f64)>,
}
impl RotationTimeline {
    pub fn empty(start_rotation: f64, duration: f32, hop_size: f32) -> Self {
        let mut samples = Vec::new();

        let mut t = 0.0;
        while t < duration {
            samples.push((t, start_rotation));
            t += hop_size;
        }

        Self { samples }
    }

    pub fn build(
        song_manager: &SongManager,
        layer_manager: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        start_rotation: f64,
        hop_size: f32,
    ) -> Self {
        let samples = (song_manager.song.duration() / hop_size) as usize;

        // build cumulative rotation timeline
        let mut cumulative = Vec::with_capacity(samples);
        let mut t = 0.0;
        let mut current_rotation = start_rotation;
        for _ in 0..samples {
            let delta_rotation = song_manager.get_rotation_change_at_timestamp(
                layer_manager.clone(),
                audio_mapper,
                t,
            );

            current_rotation += delta_rotation;

            cumulative.push((t, current_rotation));
            t += hop_size;
        }

        Self {
            samples: cumulative,
        }
    }

    pub fn sample_at(&self, timestamp: f32) -> f64 {
        if timestamp <= self.samples[0].0 {
            return self.samples[0].1;
        }
        if timestamp >= self.samples[self.samples.len() - 1].0 {
            return self.samples[self.samples.len() - 1].1;
        }

        let idx = match self
            .samples
            .binary_search_by(|(time, _)| time.partial_cmp(&timestamp).unwrap())
        {
            Ok(i) => return self.samples[i].1, // exact hit
            Err(i) => i,
        };

        let (t1, v1) = self.samples[idx - 1];
        let (t2, v2) = self.samples[idx];

        let alpha = (timestamp - t1) / (t2 - t1);
        // Interpolate angles along the shortest path to avoid sudden direction flips
        let mut delta = v2 - v1;
        delta = (delta + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI)
            - std::f64::consts::PI;
        v1 + delta * alpha as f64
    }
}

/// Responsible for managing the interpolation between [`VideoFrame`]s
pub struct VideoFrameManager {
    start_frame: VideoFrame,
    end_frame: VideoFrame,
}
impl VideoFrameManager {
    pub fn new(end_params: FractalParams) -> Self {
        Self {
            start_frame: VideoFrame::new(
                BigComplex::from_f64s(-0.5, 0.0),
                end_params.max_iterations,
            ),
            end_frame: VideoFrame::from_params(end_params),
        }
    }

    pub fn from_fractal_settings(fractal_settings: FractalSettings) -> Self {
        Self::new(fractal_settings.params)
    }

    /// `move_center_percent`: portion of animation where it should move from start -> end center
    pub fn get_frame(
        &self,
        move_center_magnification: f64,
        current_magnification: f64,
    ) -> VideoFrame {
        VideoFrame::interpolate(
            &self.start_frame,
            &self.end_frame,
            move_center_magnification,
            current_magnification,
        )
    }
}

/// Fractal render context of a frame
pub struct VideoFrame {
    center: BigComplex,
    max_iterations: u32,
}
impl VideoFrame {
    pub fn new(center: BigComplex, max_iterations: u32) -> Self {
        Self {
            center,
            max_iterations,
        }
    }

    pub fn from_params(params: FractalParams) -> Self {
        Self::new(params.center.lock().unwrap().clone(), params.max_iterations)
    }

    pub fn interpolate(
        frame1: &Self,
        frame2: &Self,
        move_center_magnification: f64,
        current_magnification: f64,
    ) -> Self {
        let center = if current_magnification < move_center_magnification {
            let ratio = current_magnification / move_center_magnification;

            // let t_f64 = ratio.powf(0.125); different slope
            let t_f64 = 1.0 - (1.0 - ratio).powi(8);

            // Convert to FBig and perform high-precision lerp for the centre
            let t_fbig = dashu_float::FBig::try_from(t_f64).unwrap_or(dashu_float::FBig::ONE);

            BigComplex::lerp(&frame1.center, &frame2.center, &t_fbig)
        } else {
            frame2.center.clone()
        };

        Self::new(center, frame1.max_iterations)
    }
}
