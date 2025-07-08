use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams,
        algorithms::render_algorithms::ReferenceOrbit,
        manager::{layer::Layer, layer_manager::LayerManager, layer_renderer::LayerRenderer},
    },
    ui::fractal_canvas::{CanvasDimensions, FractalCanvas},
};

pub struct FractalVisualiser {
    layer_manager: Arc<Mutex<LayerManager>>,
    reference_orbit: Arc<ReferenceOrbit>,
    canvas: FractalCanvas,
}
impl FractalVisualiser {
    pub fn new(params: &Arc<Mutex<FractalParams>>, canvas_dims: CanvasDimensions) -> Self {
        let layer_manager = Arc::new(Mutex::new(LayerManager::new(vec![Layer::default()], true)));
        let reference_orbit = Arc::new(ReferenceOrbit::new(params));
        let canvas = FractalCanvas::new(canvas_dims);

        Self {
            layer_manager,
            reference_orbit,
            canvas,
        }
    }

    pub fn change_dimensions(&mut self, canvas_dims: CanvasDimensions) {
        self.canvas.change_dimensions(canvas_dims);
    }

    pub fn update_render(&mut self, params: &Arc<Mutex<FractalParams>>) {
        let layer_manager = Arc::clone(&self.layer_manager);
        layer_manager
            .lock()
            .unwrap()
            .generate_palettes(params.lock().unwrap().max_iterations as f32);

        let layer_randerer = Arc::new(Mutex::new(LayerRenderer::new(layer_manager)));
        self.reference_orbit = Arc::new(ReferenceOrbit::new(params));
        self.canvas.update_render(
            layer_randerer,
            Arc::clone(&params),
            Arc::clone(&self.reference_orbit),
        );
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.canvas.draw(x, y);
    }
}
