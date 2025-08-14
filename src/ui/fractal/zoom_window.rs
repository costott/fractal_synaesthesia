use std::sync::{Arc, Mutex};

use dashu_float::FBig;
use macroquad::prelude::*;

use crate::{
    rendering::algorithms::render_algorithms::FractalParams, types::BigComplex,
    ui::fractal_canvas::CanvasDimensions,
};

struct ZoomHold {
    init_mouse_pos: Vec2,
    zoom_size: Vec2,
    center: BigComplex,
    pixel_step: f64,
}
impl ZoomHold {
    /// Proportion of the current canvas that the window can't be smaller than
    const MIN_SIZE: f64 = 0.01;

    fn apply_zoom(&self, fractal_params: Arc<Mutex<FractalParams>>) {
        let mut params = fractal_params.lock().unwrap();
        *params.center.lock().unwrap() = self.center.clone();
        params.pixel_step = self.pixel_step;
    }

    fn edit_zoom(&mut self, current_pixel_step: f64, canvas_dimensions: CanvasDimensions) {
        let (x, y) = mouse_position();
        let hold_delta = (vec2(x, y) - self.init_mouse_pos).abs();

        let zoom_fraction = (f32::max(
            2.0 * hold_delta.x / canvas_dimensions.width as f32,
            2.0 * hold_delta.y / canvas_dimensions.height as f32,
        ) as f64)
            .max(Self::MIN_SIZE);

        self.zoom_size = vec2(
            canvas_dimensions.width as f32,
            canvas_dimensions.height as f32,
        ) * zoom_fraction as f32;

        self.pixel_step = current_pixel_step * zoom_fraction;
    }
}

pub struct ZoomWindow {
    /// History of (Center, pixel_step)
    history: Vec<(BigComplex, f64)>,
    zoom_hold: Option<ZoomHold>,
}
impl ZoomWindow {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            zoom_hold: None,
        }
    }

    /// Returns if there was an update to the fractal parameters.
    pub fn update(
        &mut self,
        fractal_params: Arc<Mutex<FractalParams>>,
        canvas_dimensions: CanvasDimensions,
    ) -> bool {
        if let Some(hold) = self.zoom_hold.as_mut() {
            if !is_mouse_button_down(MouseButton::Left) {
                // release the hold
                let params = fractal_params.lock().unwrap();
                self.history
                    .push((params.center.lock().unwrap().clone(), params.pixel_step));
                drop(params);

                hold.apply_zoom(fractal_params);
                self.zoom_hold = None;
                true
            } else {
                // still holding
                let params = fractal_params.lock().unwrap();
                hold.edit_zoom(params.pixel_step, canvas_dimensions);
                false
            }
        } else {
            // start a hold
            if is_mouse_button_down(MouseButton::Left) {
                self.create_zoom_window(Arc::clone(&fractal_params), canvas_dimensions);
            }

            // go back
            if is_mouse_button_pressed(MouseButton::Right) && !self.history.is_empty() {
                let (center, pixel_step) = self.history.pop().unwrap();
                let mut params = fractal_params.lock().unwrap();
                *params.center.lock().unwrap() = center;
                params.pixel_step = pixel_step;
                return true;
            }

            false
        }
    }

    fn create_zoom_window(
        &mut self,
        fractal_params: Arc<Mutex<FractalParams>>,
        canvas_dimensions: CanvasDimensions,
    ) {
        // convert mouse pos to complex number
        let (x, y) = mouse_position();
        let params = fractal_params.lock().unwrap();
        let pixel_step = params.pixel_step;
        let big_pixel_step = FBig::try_from(pixel_step).unwrap();
        let dc = BigComplex::new(
            -FBig::try_from(canvas_dimensions.width as f32 / 2.0 - x).unwrap()
                * big_pixel_step.clone(),
            FBig::try_from(canvas_dimensions.height as f32 / 2.0 - y).unwrap() * big_pixel_step,
        );

        self.zoom_hold = Some(ZoomHold {
            init_mouse_pos: vec2(x, y),
            zoom_size: ZoomHold::MIN_SIZE as f32
                * vec2(
                    canvas_dimensions.width as f32,
                    canvas_dimensions.height as f32,
                ),
            center: params.center.lock().unwrap().clone().safe_add(&dc),
            pixel_step: pixel_step * ZoomHold::MIN_SIZE,
        });
    }

    pub fn draw(&self) {
        if !self.zoom_hold.is_some() {
            return;
        }

        let hold = self.zoom_hold.as_ref().unwrap();

        let topleft = hold.init_mouse_pos - hold.zoom_size / 2.0;
        draw_rectangle_lines(
            topleft.x,
            topleft.y,
            hold.zoom_size.x,
            hold.zoom_size.y,
            2.0,
            WHITE,
        );
    }
}
