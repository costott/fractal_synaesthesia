use std::sync::atomic::{AtomicUsize, Ordering};
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
impl CanvasDimensions {
    pub fn new_from_aspect_with_width(aspect_ratio: f32, width: u16) -> Self {
        Self {
            width,
            height: (width as f32 / aspect_ratio) as u16,
        }
    }

    pub fn new_from_aspect_with_height(aspect_ratio: f32, height: u16) -> Self {
        Self {
            width: (height as f32 * aspect_ratio) as u16,
            height,
        }
    }

    pub fn total_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn new_from_this_aspect_with_width(&self, width: u16) -> Self {
        Self::new_from_aspect_with_width(self.aspect_ratio(), width)
    }

    pub fn new_from_this_aspect_with_height(&self, height: u16) -> Self {
        Self::new_from_aspect_with_height(self.aspect_ratio(), height)
    }
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
    pub image: Arc<Mutex<Image>>,
    pub dims: CanvasDimensions,
    texture: Texture2D,
    renderer: Renderer,
    progress: Arc<AtomicUsize>,
    pub last_rendered_quality: usize,
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
            progress: Arc::new(AtomicUsize::new(0)),
            last_rendered_quality: 1,
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
        layer_renderer: &LayersRenderer,
        fractal_params: Arc<Mutex<FractalParams>>,
        reference_orbit: Arc<ReferenceOrbit>,
        quality: usize,
    ) {
        self.renderer.cancel_current_render();
        self.progress = Arc::new(AtomicUsize::new(0));
        self.last_rendered_quality = quality;

        let rx = self.renderer.spawn_render_tasks(
            self.dims,
            layer_renderer,
            fractal_params,
            reference_orbit,
            quality,
        );

        let image_clone = Arc::clone(&self.image);
        let progress_clone = Arc::clone(&self.progress);
        thread::spawn(move || {
            for (x, y, colour) in rx.iter() {
                let mut img = image_clone.lock().unwrap();
                img.set_pixel(x, y, colour);
                progress_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    pub fn get_progress(&self) -> f32 {
        let progress = self.progress.load(Ordering::Relaxed);
        progress as f32 / self.dims.total_pixels() as f32
    }

    pub fn finished_render(&self) -> bool {
        let progress = self.progress.load(Ordering::Relaxed);
        progress >= self.dims.total_pixels()
    }

    pub fn draw(&self, x: f32, y: f32) {
        self.texture.update(&self.image.lock().unwrap());
        draw_texture(&self.texture, x, y, WHITE);
    }
}
