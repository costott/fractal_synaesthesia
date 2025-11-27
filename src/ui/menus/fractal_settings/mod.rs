use std::sync::{Arc, Mutex};

use crate::{
    rendering::{
        algorithms::render_algorithms::{Fractal, FractalParams},
        manager::{
            layer::{Layer, LayerAlgorithmKind, LayerRange},
            layer_manager::LayerManager,
        },
        orbit_trap::{OrbitTrapAnalysis, OrbitTrapPoint, OrbitTrapType},
        palette::{Palette, PaletteMappingType},
    },
    types::BigComplex,
    ui::{
        AppModeScreen, FractalSettings,
        fractal::{fractal_canvas::CanvasDimensions, fractal_window::FractalWindow},
        window::WindowParams,
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
    context: FractalSettingsContext,

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
                        0.7,
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
                Layer::new(
                    LayerAlgorithmKind::Shading3D {
                        h2: 1.5,
                        angle: 45.0,
                    },
                    LayerRange::OutSet,
                    0.8,
                    Palette::default(),
                ),
                Layer::new(
                    LayerAlgorithmKind::OrbitTrap {
                        trap: OrbitTrapType::Point(OrbitTrapPoint::new(
                            (0.0, 0.0),
                            OrbitTrapAnalysis::Angle,
                        )),
                    },
                    LayerRange::InSet,
                    1.0,
                    Palette::new_even(
                        vec![WHITE, PINK, WHITE],
                        PaletteMappingType::Constant,
                        0.5,
                        0.0,
                    ),
                ),
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
            context: FractalSettingsContext {
                fractal_params: Arc::new(Mutex::new(params)),
                fractal_dims: CanvasDimensions {
                    width: 800,
                    height: 450,
                },
                rendering: false,
                request_render: false,
                update_previews: true,
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

    pub fn get_fractal_settings(&self) -> FractalSettings {
        FractalSettings {
            params: self.context.fractal_params.lock().unwrap().clone(),
            layers: self.context.layer_manager.lock().unwrap().clone(),
        }
    }
}
impl AppModeScreen for FractalSettingsMode {
    fn update(&mut self, egui_ctx: &egui::Context) -> bool {
        self.main_fractal.update(egui_ctx, &mut self.context);
        self.sidebar.update(egui_ctx, &mut self.context);
        let save_and_close = self.controls.update(egui_ctx, &mut self.context);

        save_and_close
    }

    fn draw(&self) {
        clear_background(WHITE);

        // background behind main fractal
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

#[derive(Clone)]
pub struct FractalSettingsContext {
    pub fractal_params: Arc<Mutex<FractalParams>>,
    pub fractal_dims: CanvasDimensions,
    pub rendering: bool,
    pub request_render: bool,
    pub update_previews: bool,
    pub layer_manager: Arc<Mutex<LayerManager>>,
    pub update_layers: bool,
}

pub trait FractalSettingsWindow {
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut FractalSettingsContext);
}
