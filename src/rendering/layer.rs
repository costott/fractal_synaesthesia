use std::hash::Hash;

use crate::rendering::{
    layer_algorithms::*,
    orbit_trap::OrbitTrapType,
    palette::{Palette, interpolate_colour},
};
use macroquad::prelude::*;

/// An individual rendering layer

pub struct Layer {
    pub name: String,
    pub algorithm: LayerAlgorithmKind,
    pub application_range: LayerRange,
    pub strength: f32,
    pub palette: Palette,
}
impl Layer {
    pub fn new(
        algorithm: LayerAlgorithmKind,
        application_range: LayerRange,
        strength: f32,
        palette: Palette,
    ) -> Self {
        Self {
            name: "Layer".to_owned(),
            algorithm,
            application_range,
            strength,
            palette,
        }
    }

    /// Determine the new colour of the pixel after being passed through this layer
    pub fn determine_colour(
        &self,
        prev_colour: Option<Color>,
        implementator_output: f64,
        in_set: bool,
    ) -> Option<Color> {
        if !self.application_range.layer_applies(in_set) {
            return prev_colour;
        }

        let this_colour =
            self.algorithm
                .update_colour(prev_colour, implementator_output, &self.palette);

        Some(interpolate_colour(
            match prev_colour {
                Some(c) => c,
                None => BLACK,
            },
            this_colour,
            self.strength,
        ))
    }
}
impl Default for Layer {
    fn default() -> Self {
        Self::new(
            LayerAlgorithmKind::Colour,
            LayerRange::OutSet,
            0.,
            Palette::default(),
        )
    }
}

/// The algorithm the layer uses.
pub enum LayerAlgorithmKind {
    Colour,
    OrbitTrap { trap: OrbitTrapType, shading: bool },
    Shading3D,
    TriangleInequality { shading: bool },
}
impl LayerAlgorithmKind {
    /// Returns all the different types of implementations that are reusable.
    pub fn reusable_variants() -> [Self; 3] {
        [
            Self::Colour,
            Self::Shading3D,
            Self::TriangleInequality { shading: true },
        ]
    }

    pub fn get_new_implementation(&self) -> LayerImplementation {
        match self {
            Self::Colour => LayerImplementation::Colour(ColourAlgorithm::new()),
            Self::OrbitTrap { trap, shading } => {
                LayerImplementation::OrbitTrap(OrbitTrapAlgorithm::new((*trap).clone()))
            }
            Self::Shading3D => LayerImplementation::Shading3D(Shading3DAlgorithm::new()),
            Self::TriangleInequality { shading } => {
                LayerImplementation::TriangleInequality(TriangleInequalityAlgorithm::new())
            }
        }
    }

    fn is_shading(&self) -> bool {
        match self {
            Self::Shading3D => true,
            Self::OrbitTrap { trap: _, shading } => *shading,
            Self::TriangleInequality { shading } => *shading,
            _ => false,
        }
    }

    /// Updates the pixel's colour after being passed through this layer's algorithm
    ///
    /// # Panics
    /// If this is a shading algorithm, but the pixel hasn't been given a colour to shade from previous layers.
    fn update_colour(
        &self,
        prev_colour: Option<Color>,
        implementator_output: f64,
        palette: &Palette,
    ) -> Color {
        match self {
            Self::Colour => palette.get_colour_at_layer_output(implementator_output),
            Self::OrbitTrap { trap: _, shading } | Self::TriangleInequality { shading } => {
                let out_colour = palette.get_colour_at_layer_output(implementator_output);
                if *shading {
                    interpolate_colour(prev_colour.unwrap(), out_colour, 1.0 - out_colour.r)
                } else {
                    out_colour
                }
            }
            Self::Shading3D => {
                interpolate_colour(BLACK, prev_colour.unwrap(), implementator_output as f32)
            }
        }
    }
}
impl PartialEq for LayerAlgorithmKind {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Colour => match other {
                Self::Colour => true,
                _ => false,
            },
            Self::OrbitTrap { trap, shading } => match other {
                Self::OrbitTrap {
                    trap: other_trap,
                    shading: other_shading,
                } => *trap == *other_trap && *shading == *other_shading,
                _ => false,
            },
            Self::Shading3D => match other {
                Self::Shading3D => true,
                _ => false,
            },
            Self::TriangleInequality { shading } => match other {
                Self::TriangleInequality {
                    shading: other_shading,
                } => *shading == *other_shading,
                _ => false,
            },
        }
    }
}
impl Eq for LayerAlgorithmKind {}

/// Specifies the range of the fractal set a layer is applied to.
pub enum LayerRange {
    /// Only points in the fractal set
    InSet,
    /// Only points out the fractal set
    OutSet,
    /// Points both in or out the set (every point)
    Both,
}
impl LayerRange {
    /// Returns whether they layer applies to a point in/out the set.
    pub fn layer_applies(&self, in_set: bool) -> bool {
        match self {
            LayerRange::InSet => return in_set,
            LayerRange::OutSet => return !in_set,
            LayerRange::Both => return true,
        }
    }

    // Returns whether the layer is covered by another layer already.
    fn layer_covered(&self, covered_in_set: bool, covered_out_set: bool) -> bool {
        match self {
            LayerRange::Both => covered_in_set && covered_out_set,
            LayerRange::InSet => covered_in_set,
            LayerRange::OutSet => covered_out_set,
        }
    }
}
