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
pub struct LayerRenderer {
    manager: Arc<Mutex<LayerManager>>,
    implementations: Vec<LayerImplementation>,
    implementation_map: Vec<usize>,
}
impl LayerRenderer {
    pub fn new(manager: Arc<Mutex<LayerManager>>) -> Self {
        let (implementations, implementation_map) = Self::get_implementations(&manager);
        Self {
            manager: Arc::clone(&manager),
            implementations,
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

        let mut implementations = Vec::new();
        let mut implementation_map = Vec::with_capacity(manager.layers.len());

        // Tracks if the reusable layerimplementaion for the layeralgorithmkind is already in the implementations
        let mut reusables_registry: Vec<(LayerAlgorithmKind, Option<usize>)> =
            LayerAlgorithmKind::reusable_variants()
                .into_iter()
                .map(|kind| (kind, None))
                .collect();

        for layer in &manager.layers {
            let mut index_opt = None;

            for (kind, cached_index) in &mut reusables_registry {
                if *kind == layer.algorithm {
                    index_opt = Some(cached_index.unwrap_or_else(|| {
                        let idx = implementations.len();
                        implementations.push(layer.algorithm.get_new_implementation());
                        *cached_index = Some(idx);
                        idx
                    }));
                    break;
                }
            }

            // if no index, not a reusable
            let index = index_opt.unwrap_or_else(|| {
                implementations.push(layer.algorithm.get_new_implementation());
                implementations.len() - 1
            });

            implementation_map.push(index);
        }

        (implementations, implementation_map)
    }

    /// Get the colour of the pixel by running the rendering algorithm and passing the result through all layers.
    pub fn render_pixel(
        &mut self,
        fractal: &Arc<Fractal>,
        pixel_dc: Complex,
        reference_orbit: &Arc<ReferenceOrbit>,
        max_iterations: u32,
        bailout2: f64,
    ) -> Result<Color, LayerError> {
        let in_set = analyse_pixel(
            fractal,
            pixel_dc,
            reference_orbit,
            max_iterations,
            bailout2,
            &mut self.implementations,
        );

        self.colour_pixel(in_set)
    }

    /// After determining the implementations' outputs, use it to colour the pixel by passing through all layers.
    fn colour_pixel(&self, in_set: bool) -> Result<Color, LayerError> {
        let mut colour: Option<Color> = None;
        for (i, layer) in self.manager.lock().unwrap().layers.iter().enumerate() {
            let output = self.implementations[self.implementation_map[i]].get_output();
            colour = layer.determine_colour(colour, output, in_set)?;
        }

        Ok(colour.unwrap_or(BLACK))
    }
}
