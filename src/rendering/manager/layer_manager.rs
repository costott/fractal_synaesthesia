/// Manages all the layers present for the renderer
use crate::rendering::manager::layer::Layer;
use macroquad::prelude::*;

/// Controls the structure (order, number) of the layers.
pub struct LayerManager {
    pub layers: Vec<Layer>,
}
impl LayerManager {
    pub fn new(mut layers: Vec<Layer>, overwrite_names: bool) -> Self {
        if overwrite_names {
            for (i, layer) in layers.iter_mut().enumerate() {
                layer.name = format!("Layer {}", i + 1);
            }
        }

        Self { layers }
    }

    /// Makes sure all the palettes for the layers are updated for the current max iterations
    pub fn generate_palettes(&mut self, max_iterations: f32) {
        for layer in self.layers.iter_mut() {
            layer.palette.generate_palette(max_iterations);
        }
    }
}
