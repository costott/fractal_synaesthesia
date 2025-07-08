use std::sync::{Arc, Mutex};

use macroquad::prelude::Color;

use crate::{
    rendering::{
        algorithms::render_algorithms::ReferenceOrbit, manager::layer_renderer::LayerRenderer,
        renderer::RenderParams,
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
            end_y: start_y + height,
            start_x,
            end_x: start_x + width,
        }
    }
}

/// Manages rendering in a given region
pub struct RenderTask {
    pub layer_renderer: Arc<Mutex<LayerRenderer>>,
    pub reference_orbit: Arc<ReferenceOrbit>,

    pub render_params: Arc<RenderParams>,
}
impl RenderTask {
    /// Run the render task on the given `region`, calling `emit_pixel(x, y, output_colour)` on the results.`
    pub fn run_on<F>(&self, region: TaskRegion, mut emit_pixel: F)
    where
        F: FnMut(u32, u32, Color),
    {
        for y in region.start_y..=region.end_y {
            for x in region.start_x..=region.end_x {
                let dc = Complex::new(
                    x as f64 * self.render_params.pixel_step - self.render_params.center.real_f64(),
                    y as f64 * self.render_params.pixel_step - self.render_params.center.im_f64(),
                );

                match self.layer_renderer.lock().unwrap().render_pixel(
                    &self.render_params.fractal,
                    dc,
                    &self.reference_orbit,
                    self.render_params.max_iterations,
                    self.render_params.bailout2,
                ) {
                    Ok(colour) => emit_pixel(x as u32, y as u32, colour),
                    Err(e) => {
                        emit_pixel(x as u32, y as u32, macroquad::color::RED); // fail visibly
                        eprintln!("{:?}", e);
                    }
                }
            }
        }
    }
}
