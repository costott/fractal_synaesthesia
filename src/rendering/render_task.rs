use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use macroquad::prelude::Color;

use crate::{
    rendering::{
        algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
        manager::layer_renderer::LayerRenderer,
    },
    types::Complex,
};

/// The range of `x` and `y` pixel values a [`RenderTask`] runs on.
pub struct TaskRegion {
    start_y: usize,
    end_y: usize,
    start_x: usize,
    end_x: usize,
}
impl TaskRegion {
    pub fn new(start_y: usize, height: usize, start_x: usize, width: usize) -> Self {
        Self {
            start_y,
            end_y: start_y + height - 1,
            start_x,
            end_x: start_x + width - 1,
        }
    }
}

/// Manages rendering in a given region for a thread
pub struct RenderTask {
    pub layer_renderer: Arc<Mutex<LayerRenderer>>,
    pub reference_orbit: Arc<ReferenceOrbit>,
    pub fractal_params: Arc<Mutex<FractalParams>>,
    pub cancel_render: Arc<AtomicBool>,
}
impl RenderTask {
    /// Run the render task on the given `region`, calling `emit_pixel(x, y, output_colour)` on the results.`
    pub fn run_on<F>(
        &self,
        region: TaskRegion,
        image_width: f64,
        image_height: f64,
        mut emit_pixel: F,
    ) where
        F: FnMut(u32, u32, Color),
    {
        for y in region.start_y..=region.end_y {
            for x in region.start_x..=region.end_x {
                let params = self.fractal_params.lock().unwrap();
                let pixel_step = params.pixel_step;
                let fractal = Arc::clone(&params.fractal);
                let max_iterations = params.max_iterations;
                let bailout2 = params.bailout2;
                drop(params);

                let dc = Complex::new(
                    -(image_width / 2.0 - x as f64) * pixel_step,
                    (image_height / 2.0 - y as f64) * pixel_step,
                );

                let colour = match self.layer_renderer.lock().unwrap().render_pixel(
                    &fractal,
                    dc,
                    &self.reference_orbit,
                    max_iterations,
                    bailout2,
                ) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        macroquad::color::RED // fail visibly
                    }
                };

                emit_pixel(x as u32, y as u32, colour);

                if self.cancel_render.load(Ordering::Relaxed) {
                    return;
                }
            }
        }
    }
}
