use crate::rendering::{
    algorithms::layer_algorithms::*,
    manager::layer_error::LayerError,
    orbit_trap::OrbitTrapType,
    palette::{Palette, blend_colours, interpolate_colour},
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

        let this_colour = self.get_layer_colour(prev_colour, implementator_output)?;

        Ok(Some(blend_colours(
            prev_colour.unwrap_or(BLACK),
            this_colour,
            self.strength,
        )))
    }

    fn get_layer_colour(
        &self,
        prev_colour: Option<Color>,
        implementator_output: f64,
    ) -> Result<Color, LayerError> {
        match self.algorithm.get_mapping_kind() {
            LayerMappingKind::Shade => {
                let previous = prev_colour
                    .ok_or_else(|| LayerError::ShadingError("No base colour to shade".into()))?;
                Ok(interpolate_colour(
                    BLACK,
                    previous,
                    implementator_output as f32,
                ))
            }
            LayerMappingKind::Blend => Ok(self
                .palette
                .get_colour_at_layer_output(implementator_output)),
        }
    }
}
impl Default for Layer {
    fn default() -> Self {
        Self::new(
            LayerAlgorithmKind::Colour,
            LayerRange::OutSet,
            1.,
            Palette::default(),
        )
    }
}

/// The algorithm the layer uses.
pub enum LayerAlgorithmKind {
    Colour,
    OrbitTrap {
        trap: OrbitTrapType,
    },
    Shading3D,
    TriangleInequality,
    StripeAverageAlgorithm {
        skip_iteration: u32,
        stripe_density: f64,
    },
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
            Self::TriangleInequality => {
                LayerImplementation::TriangleInequality(TriangleInequalityAlgorithm::new())
            }
            Self::StripeAverageAlgorithm {
                skip_iteration,
                stripe_density,
            } => LayerImplementation::StripeAverage(StripeAverageAlgorithm::new(
                *skip_iteration,
                *stripe_density,
            )),
        }
    }

    pub fn get_mapping_kind(&self) -> LayerMappingKind {
        match self {
            Self::Shading3D => LayerMappingKind::Shade,
            _ => LayerMappingKind::Blend,
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
            Self::StripeAverageAlgorithm {
                skip_iteration: si,
                stripe_density: sd,
            } => match other {
                Self::StripeAverageAlgorithm {
                    skip_iteration: o_si,
                    stripe_density: o_sd,
                } => si == o_si && sd == o_sd,
                _ => false,
            },
        }
    }
}
impl Eq for LayerAlgorithmKind {}

#[repr(u8)]
pub enum LayerMappingKind {
    Blend,
    Shade,
}

/// Specifies the range of the fractal set a layer is applied to.
#[repr(u8)]
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
