use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{Fractal, FractalParams},
        manager::{
            layer::{Layer, LayerAlgorithmKind, LayerRange},
            layer_manager::LayerManager,
        },
        palette::{Palette, PaletteMappingType},
    },
    types::BigComplex,
    ui::{
        AppModeScreen,
        fractal::fractal_window::FractalWindow,
        window::{Window, WindowContext, WindowParams},
    },
};
use macroquad::prelude::*;

mod controls;
use controls::Controls;
mod param_editor;
mod sidebar;
use sidebar::Sidebar;
mod layers_editor;

pub const START_PIXEL_STEP: f64 = 0.005;

pub struct FractalSettingsMode {
    window_context: WindowContext,

    main_fractal: FractalWindow,
    sidebar: Sidebar,
    controls: Controls,
}
impl FractalSettingsMode {
    pub fn new() -> Self {
        let params = FractalParams::new(
            Fractal::Mandelbrot { power: 2 },
            BigComplex::from_f64s(-0.5, 0.0),
            START_PIXEL_STEP,
            500,
            0.0,
        );

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

        Self {
            main_fractal: FractalWindow::new(
                WindowParams {
                    width: 800 as u16,
                    height: 450 as u16,
                    x: (screen_width() - (screen_width() - 500.0 + 800.0) * 0.5) as u16,
                    y: (screen_height() - (screen_height() - 150.0 + 450.0) * 0.5) as u16,
                },
                &params,
                layer_manager.clone(),
            ),
            window_context: WindowContext {
                fractal_params: Arc::new(Mutex::new(params)),
                request_render: false,
                layer_manager: layer_manager.clone(),
                update_layers: false,
            },
            sidebar: Sidebar::new(WindowParams {
                width: 500 as u16,
                height: screen_height() as u16 - 150 - 1,
                x: 0,
                y: 150 + 1,
            }),
            controls: Controls::new(WindowParams {
                width: screen_width() as u16,
                height: 150,
                x: 0,
                y: 0,
            }),
        }
    }
}
impl AppModeScreen for FractalSettingsMode {
    fn update(&mut self, egui_ctx: &egui::Context) {
        self.main_fractal.update(egui_ctx, &mut self.window_context);
        self.sidebar.update(egui_ctx, &mut self.window_context);
        self.controls.update(egui_ctx, &mut self.window_context);
    }

    fn draw(&self) {
        clear_background(WHITE);

        draw_rectangle(
            self.sidebar.params.width as f32,
            self.controls.params.height as f32,
            screen_width() - self.sidebar.params.width as f32,
            screen_height() - self.controls.params.height as f32,
            LIGHTGRAY,
        );

        self.main_fractal.draw();
    }
}
