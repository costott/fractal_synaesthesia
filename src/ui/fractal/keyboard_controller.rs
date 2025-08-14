use std::sync::{Arc, Mutex};

use dashu_float::FBig;
use macroquad::prelude::*;

use crate::rendering::algorithms::render_algorithms::FractalParams;

pub struct KeyboardController {
    move_speed: f64,
}
impl KeyboardController {
    const ZOOM_SPEED: f64 = 0.5;

    pub fn new() -> Self {
        Self { move_speed: 0.5 }
    }

    /// Returns if there was an update to the fractal params.
    pub fn update(&mut self, fractal_params: Arc<Mutex<FractalParams>>) -> bool {
        let mut updated = false;

        updated |= self.move_center(Arc::clone(&fractal_params));
        updated |= self.zoom(fractal_params);

        updated
    }

    fn move_center(&self, fractal_params: Arc<Mutex<FractalParams>>) -> bool {
        let dt = get_frame_time() as f64;
        let (mut moved_x, mut moved_y) = (true, true);
        let movement = FBig::try_from(self.move_speed * dt)
            .unwrap()
            .with_precision(0)
            .value();

        let params = fractal_params.lock().unwrap();
        let mut center_real = params.center.lock().unwrap().clone().real;
        drop(params);
        let old_real = center_real.clone();
        center_real += movement.clone()
            * FBig::try_from(match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => {
                    moved_x = false;
                    0.0
                }
            })
            .unwrap();
        // Moved but didn't have the precision to make an effect
        if moved_x && old_real == center_real {
            center_real = center_real.with_precision(old_real.precision() + 1).value();
        }

        let params = fractal_params.lock().unwrap();
        let mut center_im = params.center.lock().unwrap().clone().im;
        drop(params);
        let old_im = center_im.clone();
        center_im += movement.clone()
            * FBig::try_from(match (is_key_down(KeyCode::W), is_key_down(KeyCode::S)) {
                (true, false) => 1.0,
                (false, true) => -1.0,
                _ => {
                    moved_y = false;
                    0.0
                }
            })
            .unwrap();
        // Moved but didn't have the precision to make an effect
        if moved_y && old_im == center_im {
            center_im = center_im.with_precision(old_im.precision() + 1).value();
        }

        if moved_x || moved_y {
            let params = fractal_params.lock().unwrap();
            params.center.lock().unwrap().real = center_real;
            params.center.lock().unwrap().im = center_im;
            drop(params);
        }

        moved_x || moved_y
    }

    fn zoom(&mut self, fractal_params: Arc<Mutex<FractalParams>>) -> bool {
        let mut zoomed = true;

        let zoom_fraction = Self::ZOOM_SPEED / (get_fps() as f64 + Self::ZOOM_SPEED + 1.0);

        match (is_key_down(KeyCode::Up), is_key_down(KeyCode::Down)) {
            (true, false) => {
                fractal_params.lock().unwrap().pixel_step *= 1.0 - zoom_fraction;
                self.move_speed *= 1.0 - zoom_fraction;
            }
            (false, true) => {
                fractal_params.lock().unwrap().pixel_step *= 1.0 + zoom_fraction;
                self.move_speed *= 1.0 + zoom_fraction;
            }
            _ => zoomed = false,
        }

        zoomed
    }
}
