/// Manages all the layers present for the renderer
use crate::rendering::{
    manager::layer::{Layer, LayerAlgorithmKind, LayerRange},
    orbit_trap::{OrbitTrapAnalysis, OrbitTrapPoint, OrbitTrapType},
    palette::{Palette, PaletteMappingType},
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// Controls the structure (order, number) of the layers.
#[derive(Clone, Serialize, Deserialize)]
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

    pub fn empty() -> Self {
        Self { layers: vec![] }
    }

    /// Makes sure all the palettes for the layers are updated for the current max iterations
    pub fn generate_palettes(&mut self, max_iterations: f32) {
        for layer in self.layers.iter_mut() {
            layer.palette.generate_palette(max_iterations);
        }
    }

    pub fn add_layer(&mut self) {
        self.layers.push(Layer::default());
    }

    pub fn remove_layer(&mut self, index: usize) {
        self.layers.remove(index);
    }
}
impl Default for LayerManager {
    fn default() -> Self {
        Self::new(
            vec![
                Layer::new(
                    LayerAlgorithmKind::Colour,
                    LayerRange::OutSet,
                    1.0,
                    Palette::new_even(
                        vec![WHITE.into(), ORANGE.into(), BLUE.into(), WHITE.into()],
                        PaletteMappingType::Repeated,
                        0.1,
                        0.7,
                    ),
                ),
                Layer::new(
                    LayerAlgorithmKind::Shading3D {
                        h2: 1.5,
                        angle: 45.0,
                    },
                    LayerRange::OutSet,
                    0.8,
                    Palette::default(),
                ),
                Layer::new(
                    LayerAlgorithmKind::OrbitTrap {
                        trap: OrbitTrapType::Point(OrbitTrapPoint::new(
                            (0.0, 0.0),
                            OrbitTrapAnalysis::Angle,
                        )),
                    },
                    LayerRange::InSet,
                    1.0,
                    Palette::new_even(
                        vec![WHITE.into(), PINK.into(), WHITE.into()],
                        PaletteMappingType::Constant,
                        0.5,
                        0.0,
                    ),
                ),
            ],
            true,
        )
    }
}
