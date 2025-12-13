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
    visualiser: FractalVisualiser,
    zoom_timeline: ZoomTimeline,

    total_frames: usize,
    framerate: f32,
    hop_size: f32,
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
        audio_mapper: &AudioMapper,
        framerate: f32,
        hop_size: f32,
    ) -> Self {
        let visualiser = FractalVisualiser::new(
            params,
            context.video_dimensions,
            context.layer_manager.clone(),
            4,
            true,
        );
        let total_frames =
            (context.song_manager.as_ref().unwrap().song.duration() * framerate) as usize;

        Self {
            zoom_timeline: ZoomTimeline::build_complete(
                &context.song_manager.as_ref().unwrap(),
                visualiser.layer_manager.clone(),
                audio_mapper,
                crate::ui::menus::fractal_settings::START_PIXEL_STEP,
                params.pixel_step,
                hop_size,
            ),
            visualiser,
            frame_manager: VideoFrameManager::new(params.clone()),
            total_frames,
            framerate,
            hop_size,
        }
    }

    pub fn get_frame(
        &mut self,
        audio_mapper: &AudioMapper,
        song_manager: &SongManager,
        video_percent: f32,
    ) -> macroquad::texture::Image {
        let timestamp = self.total_frames as f32 * video_percent;

        let frame = self.frame_manager.get_frame(0.1, video_percent);

        let render_layers = song_manager.get_layers_at_timestamp(
            self.visualiser.layer_manager.clone(),
            audio_mapper,
            timestamp,
        );

        let frame_params = Arc::new(Mutex::new(FractalParams::new(
            Fractal::Mandelbrot { power: 2 },
            frame.center,
            self.zoom_timeline.sample_at(timestamp),
            frame.max_iterations,
            0.0,
        )));

        let original_layers = self.visualiser.layer_manager.clone();
        self.visualiser.layer_manager = Arc::new(Mutex::new(render_layers));

        self.visualiser.update_render(&frame_params);
        while !self.visualiser.finished_render() {}

        self.visualiser.layer_manager = original_layers;

        self.visualiser.rendered_image().lock().unwrap().clone()
    }
}

struct ZoomTimeline {
    /// (timestamp, pixel_step)
    samples: Vec<(f32, f64)>,
    final_pixel_step: f64,
}
impl ZoomTimeline {
    pub fn build_complete(
        song_manager: &SongManager,
        layer_manager: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        start_pixel_step: f64,
        final_pixel_step: f64,
        hop_size: f32,
    ) -> Self {
        Self::build_unnormalised(
            |layer_manager, timestamp| {
                song_manager.get_zoom_multiplier_at_timestamp(
                    layer_manager.clone(),
                    audio_mapper,
                    timestamp,
                )
            },
            layer_manager,
            final_pixel_step,
            song_manager.song.duration(),
            hop_size,
        )
        .integrate(start_pixel_step)
        .normalise()
    }

    pub fn build_unnormalised<F>(
        song_sampler: F,
        layer_manager: Arc<Mutex<LayerManager>>,
        final_pixel_step: f64,
        duration: f32,
        hop_size: f32,
    ) -> Self
    where
        F: Fn(Arc<Mutex<LayerManager>>, f32) -> f64,
    {
        let mut samples = Vec::new();

        let mut t = 0.0;
        while t < duration {
            let w = song_sampler(layer_manager.clone(), t);
            samples.push((t, w));
            t += hop_size;
        }

        Self {
            samples,
            final_pixel_step,
        }
    }

    pub fn integrate(mut self, start_pixel_step: f64) -> Self {
        let mut accumulated = start_pixel_step;

        for (_, w) in self.samples.iter_mut() {
            accumulated += *w;
            *w = accumulated;
        }

        self
    }

    pub fn normalise(mut self) -> Self {
        let sampled_end = self.samples.last().unwrap().1;
        let scale = self.final_pixel_step / sampled_end;

        for (_, acc) in self.samples.iter_mut() {
            *acc *= scale
        }

        self
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

        let alpha = (timestamp - t1) / (t2 - timestamp);
        v1 + (v2 - v1) * alpha as f64
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
    pub fn get_frame(&self, move_center_percent: f32, video_percent: f32) -> VideoFrame {
        assert!(0.0 <= move_center_percent && move_center_percent <= 1.0);
        assert!(0.0 <= video_percent && video_percent <= 1.0);
        VideoFrame::interpolate(
            &self.start_frame,
            &self.end_frame,
            move_center_percent,
            video_percent,
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
        move_center_percent: f32,
        video_percent: f32,
    ) -> Self {
        let center = if video_percent < move_center_percent {
            BigComplex::lerp(
                &frame1.center,
                &frame2.center,
                &dashu_float::FBig::try_from(video_percent / move_center_percent).unwrap(),
            )
        } else {
            frame2.center.clone()
        };

        Self::new(center, frame1.max_iterations)
    }
}
