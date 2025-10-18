use dashu_float::FBig;
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{
    rendering::algorithms::render_algorithms::FractalParams,
    types::{BigComplex, ComplexNumber},
    ui::fractal::fractal_canvas::CanvasDimensions,
};

const MIN_ZOOM_SIZE: f32 = 0.01;

enum ZoomWindowState {
    /// No window on screen
    Inactive,
    /// User is creating the initial window
    Creating(ZoomWindowCreator),
}

enum ZoomWindowCreatorState {
    Static,
    Resizing,
    Rotation,
}

struct ZoomWindowCreator {
    center: Vec2,
    size: Vec2,
    rotation_vector: Vec2,
    state: ZoomWindowCreatorState,
}
impl ZoomWindowCreator {
    fn new(window_rect: Rect) -> Self {
        Self {
            center: mouse_position().into(),
            size: MIN_ZOOM_SIZE * vec2(window_rect.w as f32, window_rect.h as f32),
            rotation_vector: vec2(1.0, 0.0),
            state: ZoomWindowCreatorState::Resizing,
        }
    }

    fn get_vertices(&self) -> [Vec2; 4] {
        [
            self.rotation_vector
                .rotate(Vec2::new(-self.size.x / 2.0, -self.size.y / 2.0))
                + self.center, // topleft
            self.rotation_vector
                .rotate(Vec2::new(self.size.x / 2.0, -self.size.y / 2.0))
                + self.center, // topright
            self.rotation_vector
                .rotate(Vec2::new(self.size.x / 2.0, self.size.y / 2.0))
                + self.center, // botright
            self.rotation_vector
                .rotate(Vec2::new(-self.size.x / 2.0, self.size.y / 2.0))
                + self.center, // botleft
        ]
    }

    fn get_rotation_line(&self) -> [Vec2; 2] {
        [
            self.rotation_vector
                .rotate(Vec2::new(0.0, -self.size.y / 2.0))
                + self.center,
            self.rotation_vector
                .rotate(Vec2::new(0.0, -self.size.y / 2.0 - self.size.y * 0.25))
                + self.center,
        ]
    }

    fn draw(&self) {
        let vertices = &self.get_vertices();
        for w in vertices.windows(2) {
            draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 3.0, WHITE)
        }
        draw_line(
            vertices[0].x,
            vertices[0].y,
            vertices[3].x,
            vertices[3].y,
            3.0,
            WHITE,
        );
        for v in vertices {
            draw_circle(v.x, v.y, 3.0, WHITE);
        }

        let rotation_line = self.get_rotation_line();
        draw_line(
            rotation_line[0].x,
            rotation_line[0].y,
            rotation_line[1].x,
            rotation_line[1].y,
            2.0,
            WHITE,
        );
        draw_circle(rotation_line[1].x, rotation_line[1].y, 3.0, WHITE);
    }

    fn update(&mut self, window_rect: Rect) {
        // state transisions
        match self.state {
            ZoomWindowCreatorState::Static => self.while_static(),
            _ => {
                if !is_mouse_button_down(MouseButton::Left) {
                    self.state = ZoomWindowCreatorState::Static;
                }
            }
        }

        match self.state {
            ZoomWindowCreatorState::Static => {}
            ZoomWindowCreatorState::Resizing => self.resizing(window_rect),
            ZoomWindowCreatorState::Rotation => self.rotating(),
        }
    }

    fn while_static(&mut self) {
        if !is_mouse_button_down(MouseButton::Left) {
            return;
        }

        let mouse_pos: Vec2 = mouse_position().into();

        // holding a vertex
        let vertices = self.get_vertices();
        let holding_vertex = vertices
            .into_iter()
            .map(|v| v.distance(mouse_pos))
            .reduce(f32::min)
            .and_then(|min_distance| Some(min_distance < 3.0))
            .unwrap_or(false);
        if holding_vertex {
            self.state = ZoomWindowCreatorState::Resizing;
        }

        // holding rotation line
        let line = self.get_rotation_line();
        let line_vector = line[1] - line[0];
        let mouse_vector = mouse_pos - line[0];

        // projection factor (how far along the line segment the closest point is)
        let t = f32::max(
            0.0,
            f32::min(
                1.0,
                mouse_vector.dot(line_vector) / line_vector.length_squared(),
            ),
        );

        // closest point on the line
        let projection = line[0] + line_vector * t;

        if (mouse_pos - projection).length() < 3.0 {
            self.state = ZoomWindowCreatorState::Rotation;
        }
    }

    fn resizing(&mut self, window_rect: Rect) {
        let mouse_pos: Vec2 = mouse_position().into();
        let hold_delta = mouse_pos - self.center;

        let window_fraction = f32::max(
            2.0 * hold_delta.x / window_rect.w as f32,
            2.0 * hold_delta.y / window_rect.h as f32,
        )
        .max(MIN_ZOOM_SIZE);

        self.size = vec2(window_rect.w as f32, window_rect.h as f32) * window_fraction;
    }

    fn rotating(&mut self) {
        let mouse_pos: Vec2 = mouse_position().into();
        if mouse_pos == self.center {
            return;
        }

        let hold_delta = mouse_pos - self.center;
        self.rotation_vector = vec2(0.0, 1.0).rotate(hold_delta.normalize_or(vec2(1.0, 0.0)));
    }

    fn apply_zoom(&self, window_rect: Rect, fractal_params: Arc<Mutex<FractalParams>>) {
        let params = fractal_params.lock().unwrap();
        let pixel_step = params.pixel_step;
        let big_pixel_step = FBig::try_from(pixel_step).unwrap();
        let rotation = params.rotation;
        drop(params);

        let delta = self.center - window_rect.center();
        let dc = BigComplex::new(
            FBig::try_from(delta.x).unwrap() * big_pixel_step.clone(),
            -FBig::try_from(delta.y).unwrap() * big_pixel_step,
        )
        .rotate(-rotation);

        let window_fraction = self.size.x / window_rect.w as f32;

        let params = fractal_params.lock().unwrap();
        let new_center = params.center.lock().unwrap().clone().safe_add(&dc);
        drop(params);

        let mut params = fractal_params.lock().unwrap();
        *params.center.lock().unwrap() = new_center;
        params.pixel_step *= window_fraction as f64;
        params.rotation += self.rotation_vector.to_angle() as f64;
    }
}

pub struct ZoomWindow {
    /// History of (Center, pixel_step, rotation)
    history: Vec<(BigComplex, f64, f64)>,
    state: ZoomWindowState,
}
impl ZoomWindow {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            state: ZoomWindowState::Inactive,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            ZoomWindowState::Inactive => false,
            _ => true,
        }
    }

    pub fn draw(&self) {
        match &self.state {
            ZoomWindowState::Inactive => {}
            ZoomWindowState::Creating(creator) => creator.draw(),
        }
    }

    /// Returns whether there was an update to the fractal parameters.
    ///
    /// # Arguments
    /// `window_rect`: The rectangle the fractal window is contained in on the screen.
    pub fn update(&mut self, window_rect: Rect, fractal_params: Arc<Mutex<FractalParams>>) -> bool {
        // Cancelling
        match self.state {
            ZoomWindowState::Inactive => {}
            _ => {
                if is_key_down(KeyCode::Escape) {
                    self.state = ZoomWindowState::Inactive;
                }
            }
        };

        // Updating
        match &mut self.state {
            ZoomWindowState::Inactive => self.inactive(window_rect, fractal_params),
            ZoomWindowState::Creating(creator) => {
                creator.update(window_rect);

                // Apply change
                if is_key_pressed(KeyCode::Enter) {
                    let params = fractal_params.lock().unwrap();
                    self.history.push((
                        params.center.lock().unwrap().clone(),
                        params.pixel_step,
                        params.rotation,
                    ));
                    drop(params);

                    creator.apply_zoom(window_rect, fractal_params);
                    self.state = ZoomWindowState::Inactive;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn inactive(&mut self, window_rect: Rect, fractal_params: Arc<Mutex<FractalParams>>) -> bool {
        if !window_rect.contains(mouse_position().into()) {
            return false;
        }

        // Create new window
        if is_mouse_button_down(MouseButton::Left) {
            self.state = ZoomWindowState::Creating(ZoomWindowCreator::new(window_rect));
        }

        // Undo
        if is_mouse_button_pressed(MouseButton::Right) && !self.history.is_empty() {
            let (center, pixel_step, rotation) = self.history.pop().unwrap();
            let mut params = fractal_params.lock().unwrap();
            *params.center.lock().unwrap() = center;
            params.pixel_step = pixel_step;
            params.rotation = rotation;
            return true;
        }

        false
    }
}
