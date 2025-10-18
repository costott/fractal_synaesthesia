use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{self},
};

use macroquad::{color::Color, texture::Image};
use threadpool::ThreadPool;

use crate::{
    rendering::{
        algorithms::render_algorithms::{FractalParams, ReferenceOrbit},
        manager::layers_renderer::LayersRenderer,
        render_task::{RenderTask, TaskRegion},
    },
    ui::fractal::fractal_canvas::CanvasDimensions,
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
        canvas_dimensions: CanvasDimensions,
        layer_renderer: &LayersRenderer,
        fractal_params: Arc<Mutex<FractalParams>>,
        reference_orbit: Arc<ReferenceOrbit>,
    ) -> mpsc::Receiver<(u32, u32, Color)> {
        let (tx, rx) = mpsc::channel();
        let threads = self.thread_pool.max_count();

        let image_width = canvas_dimensions.width as usize;
        let image_height = canvas_dimensions.height as usize;

        let thread_height = image_height / threads;

        for t in 0..threads {
            let render_task = Arc::new(RenderTask {
                layer_renderer: layer_renderer.clone(),
                reference_orbit: Arc::clone(&reference_orbit),
                fractal_params: Arc::clone(&fractal_params),
                cancel_render: Arc::clone(&self.cancel_render),
            });

            let tx = tx.clone();
            self.thread_pool.execute(move || {
                render_task.run_on(
                    TaskRegion::new(
                        t * thread_height,
                        if t < threads - 1 {
                            thread_height
                        } else {
                            image_height - t * thread_height // account for excess on last thread
                        },
                        0,
                        image_width,
                    ),
                    image_width as f64,
                    image_height as f64,
                    move |x, y, colour| {
                        let _ = tx.send((x, y, colour));
                    },
                );
            });
        }

        rx
    }

    pub fn cancel_current_render(&self) {
        self.cancel_render.store(true, Ordering::Relaxed);
        self.thread_pool.join();
        self.cancel_render.store(false, Ordering::Relaxed);
    }
}
