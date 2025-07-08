use std::sync::{Arc, Mutex};

use macroquad::texture::Image;
use threadpool::ThreadPool;

use crate::{
    rendering::{
        algorithms::render_algorithms::{Fractal, ReferenceOrbit},
        manager::layer_renderer::LayerRenderer,
        render_task::{RenderTask, TaskRegion},
    },
    types::BigComplex,
};

pub struct RenderParams {
    pub image: Arc<Mutex<Image>>,
    pub fractal: Arc<Fractal>,
    pub center: Arc<BigComplex>,
    pub pixel_step: f64,
    pub max_iterations: u32,
    pub bailout2: f64,
}

pub struct Renderer {
    params: Arc<RenderParams>,
    thread_pool: ThreadPool,
}
impl Renderer {
    pub fn new(params: &Arc<RenderParams>) -> Self {
        Self {
            params: Arc::clone(params),
            thread_pool: ThreadPool::new((num_cpus::get() - 1).max(1)),
        }
    }

    pub fn spawn_render_tasks(
        &self,
        layer_renderer: Arc<Mutex<LayerRenderer>>,
        reference_orbit: Arc<ReferenceOrbit>,
    ) {
        let render_task = Arc::new(RenderTask {
            layer_renderer,
            reference_orbit,
            render_params: Arc::clone(&self.params),
        });
        let threads = self.thread_pool.max_count();

        for t in 0..threads {
            let image = Arc::clone(&self.params.image);
            let render_task = Arc::clone(&render_task);

            self.thread_pool.execute(move || {
                let mut image = image.lock().unwrap();
                let thread_height = (image.height() as f32 / threads as f32).ceil() as usize;

                render_task.run_on(
                    TaskRegion::new(t * thread_height, thread_height, 0, image.width()),
                    |x, y, colour| {
                        image.set_pixel(x, y, colour);
                    },
                );
            });
        }
    }
}
