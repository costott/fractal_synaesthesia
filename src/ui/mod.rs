use eframe::egui;
use macroquad::window::{screen_height, screen_width};

use crate::{
    rendering::algorithms::render_algorithms::{Fractal, FractalParams},
    types::BigComplex,
    ui::fractal::{fractal_canvas::CanvasDimensions, fractal_window::FractalWindow},
};

pub mod fractal;
pub mod menu;

pub struct App {
    main_fractal: FractalWindow,
}
impl App {
    pub fn new() -> Self {
        Self {
            main_fractal: FractalWindow::new(
                FractalParams::new(
                    Fractal::Mandelbrot { power: 2 },
                    BigComplex::from_f64s(-0.5, 0.0),
                    0.005,
                    500,
                    0.0,
                ),
                CanvasDimensions {
                    width: screen_width() as u16,
                    height: screen_height() as u16,
                },
            ),
        }
    }

    pub fn update(&mut self) {
        self.main_fractal.update();
    }

    pub fn draw(&self) {
        self.main_fractal.draw();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test");
        });
    }
}
