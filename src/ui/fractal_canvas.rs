use std::sync::{Arc, Mutex};

use macroquad::prelude::*;

use crate::rendering::{
    algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
    manager::layer_renderer::LayerRenderer,
    renderer::Renderer,
};

pub struct CanvasDimensions {
    pub width: u16,
    pub height: u16,
}

pub struct FractalCanvas {
    image: Arc<Mutex<Image>>,
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
        &mut self,
        layer_renderer: Arc<Mutex<LayerRenderer>>,
        fractal_params: Arc<Mutex<FractalParams>>,
        reference_orbit: Arc<ReferenceOrbit>,
    ) {
        self.renderer.cancel_current_render();
        self.renderer.spawn_render_tasks(
            Arc::clone(&self.image),
            layer_renderer,
            fractal_params,
            reference_orbit,
        );
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.texture.update(&self.image.lock().unwrap());
        draw_texture(&self.texture, x, y, WHITE);
    }
}
