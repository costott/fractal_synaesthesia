use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
        manager::{layer::*, layer_manager::LayerManager, layers_renderer::LayersRenderer},
        palette::*,
    },
    ui::fractal_canvas::{CanvasDimensions, FractalCanvas},
};

use crate::rendering::orbit_trap::*;

use macroquad::prelude::*;

pub struct FractalVisualiser {
    layer_manager: Arc<Mutex<LayerManager>>,
    layer_renderer: LayersRenderer,
    reference_orbit: Arc<ReferenceOrbit>,
    pub canvas: FractalCanvas,
}
impl FractalVisualiser {
    pub fn new(params: &Arc<Mutex<FractalParams>>, canvas_dims: CanvasDimensions) -> Self {
        let layer_manager = Arc::new(Mutex::new(LayerManager::new(
            vec![
                Layer::new(
                    LayerAlgorithmKind::Colour,
                    LayerRange::OutSet,
                    1.0,
                    Palette::new_even(
                        vec![WHITE, ORANGE, BLUE, WHITE],
                        PaletteMappingType::Repeated,
                        0.1,
                        0.1,
                    ),
                ),
                // Layer::new(
                //     LayerAlgorithmKind::StripeAverageAlgorithm {
                //         skip_iteration: 1,
                //         stripe_density: 6.0,
                //     },
                //     LayerRange::OutSet,
                //     1.0,
                //     Palette::new_even(
                //         vec![RED, ORANGE, YELLOW, WHITE, ORANGE, RED],
                //         PaletteMappingType::Repeated,
                //         1.0,
                //         0.3,
                //     ),
                // ),
                // Layer::new(
                //     LayerAlgorithmKind::Shading3D {
                //         h2: 1.5,
                //         angle: 45.0,
                //     },
                //     LayerRange::OutSet,
                //     0.8,
                //     Palette::default(),
                // ),
                // Layer::new(
                //     LayerAlgorithmKind::OrbitTrap {
                //         trap: OrbitTrapType::Point(OrbitTrapPoint::new(
                //             (0.0, 0.0),
                //             OrbitTrapAnalysis::Angle,
                //         )),
                //     },
                //     LayerRange::InSet,
                //     1.0,
                //     Palette::new_even(
                //         vec![WHITE, PINK, WHITE],
                //         PaletteMappingType::Constant,
                //         0.5,
                //         0.0,
                //     ),
                // ),
            ],
            true,
        )));
        // TEMP
        layer_manager
            .lock()
            .unwrap()
            .generate_palettes(params.lock().unwrap().max_iterations as f32);
        let layer_renderer = LayersRenderer::new(Arc::clone(&layer_manager));
        // TEMP
        let reference_orbit = Arc::new(ReferenceOrbit::new(params, layer_renderer.max_bailout2));
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

    pub fn update_render(&mut self, params: &Arc<Mutex<FractalParams>>) {
        self.reference_orbit = Arc::new(ReferenceOrbit::new(
            params,
            self.layer_renderer.max_bailout2,
        ));
        self.canvas.update_render(
            &self.layer_renderer,
            Arc::clone(&params),
            Arc::clone(&self.reference_orbit),
        );
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.canvas.draw(x, y);
    }
}
