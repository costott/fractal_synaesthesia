use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
        manager::{layer::*, layer_manager::LayerManager, layers_renderer::LayersRenderer},
        palette::*,
    },
    ui::fractal::fractal_canvas::{CanvasDimensions, FractalCanvas},
};

use macroquad::prelude::*;

pub struct FractalVisualiser {
    layer_manager: Arc<Mutex<LayerManager>>,
    layer_renderer: LayersRenderer,
    reference_orbit: Arc<ReferenceOrbit>,
    pub canvas: FractalCanvas,
}
impl FractalVisualiser {
    pub fn new(
        params: &FractalParams,
        canvas_dims: CanvasDimensions,
        layer_manager: Arc<Mutex<LayerManager>>,
    ) -> Self {
        let layer_renderer = LayersRenderer::new(Arc::clone(&layer_manager));

        // create initial reference orbit
        let reference_orbit = Arc::new(ReferenceOrbit::new(
            &Arc::new(Mutex::new(params.clone())),
            layer_renderer.max_bailout2,
        ));
        let canvas = FractalCanvas::new(canvas_dims);

        Self {
            layer_manager,
            layer_renderer,
            reference_orbit,
            canvas,
        }
    }

    pub fn change_dimensions(&mut self, canvas_dims: CanvasDimensions) {
        self.canvas.change_dimensions(canvas_dims);
    }

    pub fn update_layers(&mut self) {
        self.layer_renderer = LayersRenderer::new(Arc::clone(&self.layer_manager));
    }

    pub fn update_render(&mut self, params: &Arc<Mutex<FractalParams>>) {
        self.reference_orbit = Arc::new(ReferenceOrbit::new(
            params,
            self.layer_renderer.max_bailout2,
        ));
        self.layer_manager
            .lock()
            .unwrap()
            .generate_palettes(params.lock().unwrap().max_iterations as f32);

        self.canvas.update_render(
            &self.layer_renderer,
            Arc::clone(&params),
            Arc::clone(&self.reference_orbit),
        );
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
