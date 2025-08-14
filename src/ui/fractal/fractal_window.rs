use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams, fractal_visualiser::FractalVisualiser,
    },
    ui::fractal::{fractal_canvas::CanvasDimensions, zoom_window_new::ZoomWindow},
};

/// A UI element for a fractal
pub struct FractalWindow {
    fractal_visualiser: FractalVisualiser,
    fractal_params: Arc<Mutex<FractalParams>>,
    zoom_window: ZoomWindow,
}
impl FractalWindow {
    pub fn new(params: FractalParams, dimensions: CanvasDimensions) -> Self {
        let params = Arc::new(Mutex::new(params));

        let mut visualiser = FractalVisualiser::new(&Arc::clone(&params), dimensions);
        visualiser.update_render(&Arc::clone(&params));

        Self {
            fractal_params: params,
            fractal_visualiser: visualiser,
            zoom_window: ZoomWindow::new(),
        }
    }

    pub fn update(&mut self) {
        let changed = self.zoom_window.update(
            Arc::clone(&self.fractal_params),
            self.fractal_visualiser.canvas.dims,
        );

        if changed {
            self.fractal_visualiser
                .update_render(&Arc::clone(&self.fractal_params));
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        self.fractal_visualiser.draw(0.0, 0.0);
        self.zoom_window.draw();
    }
}
