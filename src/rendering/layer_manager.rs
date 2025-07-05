/// Manages all the layers present for the renderer
use crate::{
    rendering::{
        layer::{Layer, LayerAlgorithmKind},
        layer_algorithms::{LayerAlgorithm, LayerImplementation},
        render_algorithms::{Fractal, ReferenceOrbit, analyse_pixel},
    },
    types::*,
};
use macroquad::prelude::*;

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

    /// Creates the implementators to use for rendering a pixel. Should be created before rendering a pixel batch as it will
    /// start the same for all pixels in the same render pass.
    ///
    /// # Returns
    /// A tuple consisting of:
    /// * The vector of starting [`LayerImplementation`]s
    /// * A vector which maps, for every index of the layers, a value representing the [`LayerImplementation`] output to use.
    fn get_implementations(&self) -> (Vec<LayerImplementation>, Vec<usize>) {
        let mut implementations = Vec::new();
        let mut implementation_map = Vec::with_capacity(self.layers.len());
        // Tracks if the reusable layerimplementaion for the layeralgorithmkind is already in the implementations
        let mut reusables_in: Vec<(LayerAlgorithmKind, i16)> = Vec::new();
        for imp in LayerAlgorithmKind::reusable_variants() {
            reusables_in.push((imp, -1));
        }

        for layer in self.layers.iter() {
            let mut added = false;
            for (algorithm_kind, index) in reusables_in.iter_mut() {
                if *algorithm_kind == layer.algorithm {
                    if *index == -1 {
                        implementations.push(layer.algorithm.get_new_implementation());
                        *index = (implementations.len() - 1) as i16;
                    }
                    implementation_map.push(*index as usize);
                    added = true;
                    break;
                }
            }
            // Not a reusable implementator
            if !added {
                implementations.push(layer.algorithm.get_new_implementation());
                implementation_map.push(implementations.len() - 1);
            }
        }

        (implementations, implementation_map)
    }

    /// Get the colour of the pixel by running the rendering algorithm and passing the result through all layers.
    pub fn render_pixel(
        &self,
        fractal: &Fractal,
        pixel_dc: Complex,
        reference_orbit: &ReferenceOrbit,
        max_iterations: u32,
        bailout2: f64,
        mut implementations: Vec<LayerImplementation>,
        implementation_map: Vec<usize>,
    ) -> Color {
        let in_set = analyse_pixel(
            fractal,
            pixel_dc,
            reference_orbit,
            max_iterations,
            bailout2,
            &mut implementations,
        );

        self.colour_pixel(&implementations, &implementation_map, in_set)
    }

    /// After determining the implementations' outputs, use it to colour the pixel by passing through all layers.
    fn colour_pixel(
        &self,
        implementations: &Vec<LayerImplementation>,
        implementation_map: &Vec<usize>,
        in_set: bool,
    ) -> Color {
        let mut colour: Option<Color> = None;
        for (i, layer) in self.layers.iter().enumerate() {
            let output = implementations[implementation_map[i]].get_output();
            colour = layer.determine_colour(colour, output, in_set);
        }

        match colour {
            Some(c) => c,
            None => BLACK,
        }
    }
}
