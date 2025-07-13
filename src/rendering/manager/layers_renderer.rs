use std::sync::{Arc, Mutex};

use macroquad::prelude::*;

use crate::{
    rendering::{
        algorithms::{
            layer_algorithms::{LayerAlgorithm, LayerImplementation},
            render_algorithms::{Fractal, ReferenceOrbit, analyse_pixel},
        },
        manager::{
            layer::LayerAlgorithmKind, layer_error::LayerError, layer_manager::LayerManager,
        },
    },
    types::Complex,
};

/// Handles rendering logic for the set of layers
#[derive(Clone)]
pub struct LayersRenderer {
    manager: Arc<Mutex<LayerManager>>,
    start_implementations: Vec<LayerImplementation>,
    pub max_bailout2: f64,
    implementation_map: Vec<usize>,
}
impl LayersRenderer {
    pub fn new(manager: Arc<Mutex<LayerManager>>) -> Self {
        let (implementations, implementation_map) = Self::get_implementations(&manager);
        let max_bailout2 = implementations
            .iter()
            .map(|im| im.get_bailout2())
            .reduce(f64::max)
            .unwrap_or(0.0);

        Self {
            manager: Arc::clone(&manager),
            start_implementations: implementations,
            max_bailout2,
            implementation_map,
        }
    }

    /// Creates the implementators to use for rendering a pixel. Should be created before rendering a pixel batch as it will
    /// start the same for all pixels in the same render pass.
    ///
    /// # Returns
    /// A tuple consisting of:
    /// * The vector of starting [`LayerImplementation`]s
    /// * A vector which maps, for every index of the layers, a value representing the [`LayerImplementation`] output to use.
    fn get_implementations(
        manager: &Arc<Mutex<LayerManager>>,
    ) -> (Vec<LayerImplementation>, Vec<usize>) {
        let manager = manager.lock().unwrap();

        let mut kinds = Vec::new();
        let mut implementations = Vec::new();
        let mut implementation_map = Vec::with_capacity(manager.layers.len());

        for layer in &manager.layers {
            let mut index_opt = None;

            // Check if we can use a previous implementation
            for (idx, kind) in kinds.iter().enumerate() {
                if *kind == layer.algorithm {
                    index_opt = Some(idx);
                    break;
                }
            }

            // if no index, not used before
            let index = index_opt.unwrap_or_else(|| {
                kinds.push(layer.algorithm.clone());
                implementations.push(layer.algorithm.get_new_implementation());
                implementations.len() - 1
            });

            implementation_map.push(index);
        }

        (implementations, implementation_map)
    }

    /// Get the colour of the pixel by running the rendering algorithm and passing the result through all layers.
    pub fn render_pixel(
        &self,
        fractal: &Arc<Fractal>,
        pixel_dc: Complex,
        reference_orbit: &Arc<ReferenceOrbit>,
        max_iterations: u32,
    ) -> Result<Color, LayerError> {
        let mut this_implementations = self.start_implementations.clone();

        analyse_pixel(
            fractal,
            pixel_dc,
            reference_orbit,
            max_iterations,
            self.max_bailout2,
            &mut this_implementations,
        );

        self.colour_pixel(&this_implementations)
    }

    /// After determining the implementations' outputs, use it to colour the pixel by passing through all layers.
    fn colour_pixel(
        &self,
        implementations: &Vec<LayerImplementation>,
    ) -> Result<Color, LayerError> {
        let mut colour: Option<Color> = None;
        for (i, layer) in self.manager.lock().unwrap().layers.iter().enumerate() {
            let implementation = &implementations[self.implementation_map[i]];
            let output = implementation.get_output();
            colour = layer.determine_colour(colour, output, implementation.get_in_set())?;
        }

        Ok(colour.unwrap_or(BLACK))
    }
}
