use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams, fractal_visualiser::FractalVisualiser,
        manager::layer_manager::LayerManager,
    },
    ui::{
        WindowParams, fractal::zoom_window::ZoomWindow,
        menus::fractal_settings::FractalSettingsWindow,
    },
};

/// A UI element for a fractal
pub struct FractalWindow {
    params: WindowParams,

    fractal_visualiser: FractalVisualiser,
    zoom_window: ZoomWindow,

    /// Used for initial render
    initialised: bool,
}
impl FractalWindow {
    pub fn new(
        window_params: WindowParams,
        fractal_params: Arc<Mutex<FractalParams>>,
        layer_manager: Arc<Mutex<LayerManager>>,
    ) -> Self {
        Self {
            fractal_visualiser: FractalVisualiser::new(
                fractal_params,
                (window_params.width, window_params.height).into(),
                layer_manager,
                4,
                true,
            ),
            params: window_params,
            zoom_window: ZoomWindow::new(),
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
impl FractalSettingsWindow for FractalWindow {
    fn update(
        &mut self,
        _egui_ctx: &egui::Context,
        project: &mut crate::project::Project,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) {
        self.fractal_visualiser.improve_quality();

        let changed = self.zoom_window.update(
            self.params.get_bounding_rect(),
            project.fractal_settings.params.clone(),
            &mut ctx.zoom_window_state,
        );

        if ctx.update_layers {
            self.fractal_visualiser.update_layers();
            ctx.update_layers = false;
        }

        if ctx.rendering && self.fractal_visualiser.finished_render() {
            ctx.rendering = false;
        }

        if changed || !self.initialised || ctx.request_render {
            self.fractal_visualiser
                .update_render(project.fractal_settings.params.clone());
            self.initialised = true;
            ctx.rendering = true;
            ctx.request_render = false;
            ctx.update_previews = true;
        }
    }
}
