pub mod fractal_canvas;
pub mod keyboard_controller;
pub mod zoom_window;

use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{Fractal, FractalParams},
        fractal_visualiser::FractalVisualiser,
    },
    types::BigComplex,
    ui::{fractal_canvas::CanvasDimensions, zoom_window::ZoomWindow},
};

pub struct App {
    fractal_visualiser: FractalVisualiser,
    fractal_params: Arc<Mutex<FractalParams>>,
    zoom_window: ZoomWindow,
}
impl App {
    pub fn new() -> Self {
        let params = Arc::new(Mutex::new(FractalParams {
            fractal: Arc::new(Fractal::Mandelbrot { power: 2 }),
            center: Arc::new(Mutex::new(BigComplex::from_f64s(-0.5, 0.0))),
            pixel_step: 0.005,
            max_iterations: 500,
            bailout2: 1e8,
        }));

        let dims = CanvasDimensions {
            width: screen_width() as u16,
            height: screen_height() as u16,
        };

        let mut visualiser = FractalVisualiser::new(&Arc::clone(&params), dims);
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
