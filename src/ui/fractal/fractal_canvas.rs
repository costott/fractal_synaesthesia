use std::sync::{Arc, Mutex};
use std::thread;

use macroquad::prelude::*;

use crate::rendering::{
    algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
    manager::layers_renderer::LayersRenderer,
    renderer::Renderer,
};

#[derive(Clone, Copy)]
pub struct CanvasDimensions {
    pub width: u16,
    pub height: u16,
}
impl From<(u16, u16)> for CanvasDimensions {
    fn from(value: (u16, u16)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

/// UI canvas to render the fractal to
pub struct FractalCanvas {
    image: Arc<Mutex<Image>>,
    pub dims: CanvasDimensions,
    texture: Texture2D,
    renderer: Renderer,
}
impl FractalCanvas {
    pub fn new(dimensions: CanvasDimensions) -> Self {
        let image = Arc::new(Mutex::new(Image::gen_image_color(
            dimensions.width,
            dimensions.height,
            BLANK,
        )));
        let texture = Texture2D::from_image(&image.lock().unwrap());

        let renderer = Renderer::new();

        Self {
            image,
            dims: dimensions,
            texture,
            renderer,
        }
    }

    pub fn change_dimensions(&mut self, dimensions: CanvasDimensions) {
        self.image = Arc::new(Mutex::new(Image::gen_image_color(
            dimensions.width,
            dimensions.height,
            BLANK,
        )));
        self.texture = Texture2D::from_image(&self.image.lock().unwrap());
    }

    pub fn update_render(
        &self,
        layer_renderer: &LayersRenderer,
        fractal_params: Arc<Mutex<FractalParams>>,
        reference_orbit: Arc<ReferenceOrbit>,
    ) {
        self.renderer.cancel_current_render();
        let rx = self.renderer.spawn_render_tasks(
            self.dims,
            layer_renderer,
            fractal_params,
            reference_orbit,
        );

        let image_clone = Arc::clone(&self.image);
        thread::spawn(move || {
            for (x, y, colour) in rx.iter() {
                let mut img = image_clone.lock().unwrap();
                img.set_pixel(x, y, colour);
            }
        });
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.texture.update(&self.image.lock().unwrap());
        draw_texture(&self.texture, x, y, WHITE);
    }
}
