use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams, fractal_visualiser::FractalVisualiser,
        manager::layer_manager::LayerManager,
    },
    ui::{Window, WindowContext, WindowParams, fractal::zoom_window::ZoomWindow},
};

/// A UI element for a fractal
pub struct FractalWindow {
    params: WindowParams,

    fractal_visualiser: FractalVisualiser,
    zoom_window: ZoomWindow,
    is_open: bool,

    /// Used for initial render
    initialised: bool,
}
impl FractalWindow {
    pub fn new(
        window_params: WindowParams,
        fractal_params: &FractalParams,
        layer_manager: Arc<Mutex<LayerManager>>,
    ) -> Self {
        Self {
            fractal_visualiser: FractalVisualiser::new(
                fractal_params,
                (window_params.width, window_params.height).into(),
                layer_manager,
            ),
            params: window_params,
            zoom_window: ZoomWindow::new(),
            is_open: true,
            initialised: false,
        }
    }

    pub fn draw(&self) {
        draw_rectangle(
            self.params.x as f32 - 1.0,
            self.params.y as f32 - 1.0,
            self.params.width as f32 + 2.0,
            self.params.height as f32 + 2.0,
            BLACK,
        );
        self.fractal_visualiser
            .draw(self.params.x as f32, self.params.y as f32);

        self.zoom_window.draw();
    }
}
impl Window for FractalWindow {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }

    fn update(&mut self, _egui_ctx: &egui::Context, ctx: &mut WindowContext) {
        let changed = self.zoom_window.update(
            self.params.get_bounding_rect(),
            Arc::clone(&ctx.fractal_params),
        );

        if ctx.update_layers {
            self.fractal_visualiser.update_layers();
        }

        if changed || !self.initialised || ctx.request_render {
            self.fractal_visualiser
                .update_render(&Arc::clone(&ctx.fractal_params));
            self.initialised = true;
            ctx.request_render = false;
        }
    }
}
