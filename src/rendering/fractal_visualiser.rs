use std::{
    cmp::{max, min},
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    rendering::{
        algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
        manager::{layer::*, layer_manager::LayerManager, layers_renderer::LayersRenderer},
        palette::*,
    },
    ui::fractal::fractal_canvas::{CanvasDimensions, FractalCanvas},
};

use macroquad::prelude::*;

struct DynamicQualityManager {
    start_of_render: Instant,
}
impl DynamicQualityManager {
    const TARGET_INITIAL: f64 = 0.2; // desired time for an initial render (seconds)
    const TARGET_INCREMENTAL: f64 = 0.1; // desired time for each incremental render (seconds)
    const MIN_QUALITY: usize = 1;
    const MAX_QUALITY: usize = 256;

    pub fn new() -> Self {
        Self {
            start_of_render: Instant::now(),
        }
    }

    pub fn start(&mut self) {
        self.start_of_render = Instant::now();
    }

    pub fn finished_incremental_render(&self, next_quality: &mut usize) {
        let elapsed = self.start_of_render.elapsed().as_secs_f64();
        let ratio = elapsed / Self::TARGET_INCREMENTAL;
        let q = *next_quality;

        // If we've already hit the highest fidelity, stop
        if q <= Self::MIN_QUALITY {
            *next_quality = Self::MIN_QUALITY;
            return;
        }

        // Very fast previous incremental render, drop quality considerably so we get to higher fidelity faster
        if ratio < 0.5 {
            *next_quality = max(q / 4, Self::MIN_QUALITY);
            return;
        }

        // Moderately fast, halve the downsampling (moderate step)
        if ratio < 1.0 {
            *next_quality = max(q / 2, Self::MIN_QUALITY);
            return;
        }

        // Otherwise just reduce quality by 1 for slow incremental render
        *next_quality = max(q.saturating_sub(1), Self::MIN_QUALITY);
    }

    pub fn while_initial_render(&self, progress: f32, visualiser_quality: &mut usize) -> bool {
        let elapsed = self.start_of_render.elapsed().as_secs_f64();
        if elapsed > Self::TARGET_INITIAL * 1.5 && progress < 0.7 {
            // If we've been rendering for more than double the target time, increase quality to speed up
            // increase quality based on how far we are through the render
            let mut q = *visualiser_quality;
            q += ((1.0 - progress) * 10.0).ceil() as usize;
            *visualiser_quality = min(q, Self::MAX_QUALITY);
            return true;
        }
        false
    }
}

pub struct FractalVisualiser {
    pub layer_manager: Arc<Mutex<LayerManager>>,
    layer_renderer: LayersRenderer,
    reference_orbit: Arc<ReferenceOrbit>,
    pub canvas: FractalCanvas,

    quality: usize,
    dynamic_quality_manager: Option<DynamicQualityManager>,
    rendering_params: Option<Arc<Mutex<FractalParams>>>,
}
impl FractalVisualiser {
    pub fn new(
        params: &FractalParams,
        canvas_dims: CanvasDimensions,
        layer_manager: Arc<Mutex<LayerManager>>,
        quality: usize,
        dynamic_quality: bool,
    ) -> Self {
        let layer_renderer = LayersRenderer::new(Arc::clone(&layer_manager));

        // create initial reference orbit
        let reference_orbit = Arc::new(ReferenceOrbit::new(
            Arc::new(Mutex::new(params.clone())),
            layer_renderer.max_bailout2,
        ));
        let canvas = FractalCanvas::new(canvas_dims);

        Self {
            layer_manager,
            layer_renderer,
            reference_orbit,
            canvas,
            quality,
            dynamic_quality_manager: if dynamic_quality {
                Some(DynamicQualityManager::new())
            } else {
                None
            },
            rendering_params: None,
        }
    }

    pub fn change_dimensions(&mut self, canvas_dims: CanvasDimensions) {
        self.canvas.change_dimensions(canvas_dims);
    }

    pub fn update_layers(&mut self) {
        self.layer_renderer = LayersRenderer::new(Arc::clone(&self.layer_manager));
    }

    pub fn update_render(&mut self, params: Arc<Mutex<FractalParams>>) {
        self.rendering_params = Some(Arc::clone(&params));
        self.reference_orbit = Arc::new(ReferenceOrbit::new(
            Arc::clone(&params),
            self.layer_renderer.max_bailout2,
        ));
        self.layer_manager
            .lock()
            .unwrap()
            .generate_palettes(params.lock().unwrap().max_iterations as f32);

        self.update_layers();

        self.start_canvas_render(self.quality);
    }

    fn start_canvas_render(&mut self, quality: usize) {
        if self.rendering_params.is_none() {
            return;
        }

        if let Some(m) = self.dynamic_quality_manager.as_mut() {
            m.start();
        }
        self.canvas.update_render(
            &self.layer_renderer,
            Arc::clone(&self.rendering_params.as_ref().unwrap()),
            Arc::clone(&self.reference_orbit),
            quality,
        );
    }

    /// Attempt to improve the quality of the current render if possible
    ///
    /// Ensure that this is only called when a render is in progress
    pub fn improve_quality(&mut self) {
        if !self.finished_render() {
            if self.canvas.last_rendered_quality != self.quality {
                // already rendering at a different quality, wait for that to finish
                return;
            }

            if let Some(m) = self.dynamic_quality_manager.as_mut() {
                // check if we need to increase quality to speed up rendering
                if m.while_initial_render(self.canvas.get_progress(), &mut self.quality) {
                    self.start_canvas_render(self.quality);
                }
            }

            return;
        }

        if self.canvas.last_rendered_quality == 1 || self.dynamic_quality_manager.is_none() {
            return;
        }

        let mut new_quality = self.canvas.last_rendered_quality;
        if let Some(m) = self.dynamic_quality_manager.as_ref() {
            m.finished_incremental_render(&mut new_quality);
        }

        self.start_canvas_render(new_quality);
    }

    pub fn get_progress(&self) -> f32 {
        self.canvas.get_progress()
    }

    pub fn finished_render(&self) -> bool {
        self.canvas.finished_render()
    }

    pub fn rendered_image(&self) -> Arc<Mutex<Image>> {
        self.canvas.image.clone()
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.canvas.draw(x, y);
    }
}
