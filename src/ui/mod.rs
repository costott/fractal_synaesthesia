use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::algorithms::render_algorithms::{Fractal, FractalParams},
    types::BigComplex,
    ui::{fractal::fractal_window::FractalWindow, menus::param_editor::ParamEditor, window::*},
};

pub mod fractal;
mod menus;
mod window;

pub const NORMAL_TEXT_SIZE: f32 = 15.0;
pub const START_PIXEL_STEP: f64 = 0.005;

pub struct App {
    main_fractal: FractalWindow,
    window_context: WindowContext,

    // TEMP
    param_editor: ParamEditor,
}
impl App {
    pub fn new() -> Self {
        let params = FractalParams::new(
            Fractal::Mandelbrot { power: 2 },
            BigComplex::from_f64s(-0.5, 0.0),
            START_PIXEL_STEP,
            500,
            0.0,
        );

        Self {
            main_fractal: FractalWindow::new(
                WindowParams {
                    width: 500 as u16,
                    height: 500 as u16,
                    x: screen_width() as u16 - 500,
                    y: screen_height() as u16 - 500,
                },
                &params,
            ),
            window_context: WindowContext {
                fractal_params: Arc::new(Mutex::new(params)),
                request_render: false,
            },
            param_editor: ParamEditor::new(WindowParams {
                width: 500 as u16,
                height: screen_height() as u16,
                x: 0,
                y: 0,
            }),
        }
    }

    pub fn update(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.style_mut(|style| {
                style.visuals.override_text_color = Some(egui::Color32::DARK_GRAY);
                style.visuals.extreme_bg_color = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.inactive.bg_fill = egui::Color32::LIGHT_GRAY;
                style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            });

            self.main_fractal.update(egui_ctx, &mut self.window_context);
            self.param_editor.update(egui_ctx, &mut self.window_context);
        });
    }

    pub fn draw(&self) {
        clear_background(WHITE);
        self.main_fractal.draw();
        egui_macroquad::draw();
    }
}
