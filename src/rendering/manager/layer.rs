use crate::rendering::{
    algorithms::layer_algorithms::*,
    manager::layer_error::LayerError,
    orbit_trap::OrbitTrapType,
    palette::{Palette, blend_colours},
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
        assert!(0.0 <= strength && strength <= 1.0);

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
    ) -> Result<Option<Color>, LayerError> {
        if !self.application_range.layer_applies(in_set) {
            return Ok(prev_colour);
        }

        let this_colour = self
            .palette
            .get_colour_at_layer_output(implementator_output);

        Ok(Some(blend_colours(
            prev_colour.unwrap_or(BLACK),
            this_colour,
            self.strength,
        )))
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
    OrbitTrap { trap: OrbitTrapType },
    Shading3D,
    TriangleInequality,
}
impl LayerAlgorithmKind {
    /// Returns all the different types of implementations that are reusable.
    pub fn reusable_variants() -> [Self; 3] {
        [Self::Colour, Self::Shading3D, Self::TriangleInequality]
    }

    pub fn get_new_implementation(&self) -> LayerImplementation {
        match self {
            Self::Colour => LayerImplementation::Colour(ColourAlgorithm::new()),
            Self::OrbitTrap { trap, .. } => {
                LayerImplementation::OrbitTrap(OrbitTrapAlgorithm::new((*trap).clone()))
            }
            Self::Shading3D => LayerImplementation::Shading3D(Shading3DAlgorithm::new()),
            Self::TriangleInequality { .. } => {
                LayerImplementation::TriangleInequality(TriangleInequalityAlgorithm::new())
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
            Self::OrbitTrap { trap } => match other {
                Self::OrbitTrap { trap: other_trap } => *trap == *other_trap,
                _ => false,
            },
            Self::Shading3D => match other {
                Self::Shading3D => true,
                _ => false,
            },
            Self::TriangleInequality => match other {
                Self::TriangleInequality => true,
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
            LayerRange::InSet => in_set,
            LayerRange::OutSet => !in_set,
            LayerRange::Both => true,
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
