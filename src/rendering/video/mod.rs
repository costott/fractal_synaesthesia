use crate::{
    rendering::{
        algorithms::render_algorithms::FractalParams, manager::layer_manager::LayerManager,
    },
    ui::FractalSettings,
};

pub struct VideoManager {
    layers: LayerManager,
    end_frame: VideoFrame,
}
impl VideoManager {
    pub fn new(layers: LayerManager, params: FractalParams) -> Self {
        Self {
            layers,
            end_frame: VideoFrame::new(params),
        }
    }

    pub fn from_fractal_settings(fractal_settings: FractalSettings) -> Self {
        Self::new(fractal_settings.layers, fractal_settings.params)
    }

    // pub fn get_frame(timestamp: f32) -> ?
}

struct VideoFrame {
    params: FractalParams,
}
impl VideoFrame {
    pub fn new(params: FractalParams) -> Self {
        Self { params }
    }
}
