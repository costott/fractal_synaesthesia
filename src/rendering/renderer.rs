use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use macroquad::texture::Image;
use threadpool::ThreadPool;

use crate::rendering::{
    algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
    manager::layers_renderer::LayersRenderer,
    render_task::{RenderTask, TaskRegion},
};

pub struct Renderer {
    thread_pool: ThreadPool,
    cancel_render: Arc<AtomicBool>,
}
impl Renderer {
    pub fn new() -> Self {
        Self {
            thread_pool: ThreadPool::new(num_cpus::get_physical() - 1),
            cancel_render: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_render_tasks(
        &self,
        image: Arc<Mutex<Image>>,
        layer_renderer: &LayersRenderer,
        fractal_params: Arc<Mutex<FractalParams>>,
        reference_orbit: Arc<ReferenceOrbit>,
    ) {
        let threads = self.thread_pool.max_count();

        for t in 0..threads {
            let render_task = Arc::new(RenderTask {
                layer_renderer: layer_renderer.clone(),
                reference_orbit: Arc::clone(&reference_orbit),
                fractal_params: fractal_params.lock().unwrap().clone(),
                cancel_render: Arc::clone(&self.cancel_render),
            });
            let image = Arc::clone(&image);

            self.thread_pool.execute(move || {
                let image_height = image.lock().unwrap().height();
                let mut thread_height = image_height / threads;
                // account for excess on last thread
                if t == threads - 1 {
                    thread_height = image_height - t * thread_height - 1;
                }

                let image_width = image.lock().unwrap().width();
                render_task.run_on(
                    TaskRegion::new(t * thread_height, thread_height, 0, image_width),
                    image_width as f64,
                    image_height as f64,
                    |x, y, colour| {
                        image.lock().unwrap().set_pixel(x, y, colour);
                    },
                );
            });
        }
    }

    pub fn cancel_current_render(&mut self) {
        self.cancel_render.store(true, Ordering::Relaxed);
        self.thread_pool.join();
        self.cancel_render.store(false, Ordering::Relaxed);
    }
}
