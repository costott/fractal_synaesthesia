use crate::rendering::{
    algorithms::layer_algorithms::*,
    manager::layer_error::LayerError,
    orbit_trap::OrbitTrapType,
    palette::{Palette, blend_colours},
};
use crate::types::colour::*;

/// An individual rendering layer
#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
            name: "New layer".to_owned(),
            algorithm,
            application_range,
            strength,
            palette,
        }
    }

    /// Determine the new colour of the pixel after being passed through this layer
    pub fn determine_colour(
        &self,
        prev_colour: Option<Colour>,
        implementator_output: f64,
        in_set: bool,
    ) -> Result<Option<Colour>, LayerError> {
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
        prev_colour: Option<Colour>,
        implementator_output: f64,
    ) -> Result<Colour, LayerError> {
        match self.algorithm.get_mapping_kind() {
            LayerMappingKind::Shade => {
                let previous = prev_colour
                    .ok_or_else(|| LayerError::ShadingError("No base colour to shade".into()))?;
                Ok(BLACK.interpolate(&previous, implementator_output as f32))
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
            LayerAlgorithmKind::Colour {
                bailout2: ColourAlgorithm::DEFAULT_BAILOUT2,
            },
            LayerRange::OutSet,
            0.,
            Palette::default(),
        )
    }
}

/// The algorithm the layer uses.
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayerAlgorithmKind {
    Colour {
        bailout2: f64,
    },
    OrbitTrap {
        bailout2: f64,
        trap: OrbitTrapType,
    },
    Shading3D {
        bailout2: f64,
        h2: f64,
        angle: f64,
    },
    TriangleInequality {
        bailout2: f64,
        apower: f64,
    },
    StripeAverage {
        bailout2: f64,
        skip_iteration: u32,
        stripe_density: f64,
    },
}
impl LayerAlgorithmKind {
    pub fn get_new_implementation(&self) -> LayerImplementation {
        match self {
            Self::Colour { bailout2 } => {
                LayerImplementation::Colour(ColourAlgorithm::new(*bailout2))
            }
            Self::OrbitTrap { bailout2, trap } => {
                LayerImplementation::OrbitTrap(OrbitTrapAlgorithm::new((*trap).clone(), *bailout2))
            }
            Self::Shading3D {
                bailout2,
                h2,
                angle,
            } => LayerImplementation::Shading3D(Shading3DAlgorithm::new(*h2, *angle, *bailout2)),
            Self::TriangleInequality { bailout2, apower } => {
                LayerImplementation::TriangleInequality(TriangleInequalityAlgorithm::new(
                    *apower, *bailout2,
                ))
            }
            Self::StripeAverage {
                bailout2,
                skip_iteration,
                stripe_density,
            } => LayerImplementation::StripeAverage(StripeAverageAlgorithm::new(
                *skip_iteration,
                *stripe_density,
                *bailout2,
            )),
        }
    }

    pub fn get_mapping_kind(&self) -> LayerMappingKind {
        match self {
            Self::Shading3D { .. } => LayerMappingKind::Shade,
            _ => LayerMappingKind::Blend,
        }
    }
}
impl crate::ui::Dropdown<LayerAlgorithmKind> for LayerAlgorithmKind {
    fn get_variants() -> Vec<LayerAlgorithmKind> {
        vec![
            Self::Colour {
                bailout2: ColourAlgorithm::DEFAULT_BAILOUT2,
            },
            Self::OrbitTrap {
                bailout2: OrbitTrapAlgorithm::DEFAULT_BAILOUT2,
                trap: OrbitTrapType::default(),
            },
            Self::Shading3D {
                bailout2: Shading3DAlgorithm::DEFAULT_BAILOUT2,
                h2: Shading3DAlgorithm::DEFAULT_H2,
                angle: Shading3DAlgorithm::DEFAULT_ANGLE,
            },
            Self::TriangleInequality {
                bailout2: TriangleInequalityAlgorithm::DEFAULT_BAILOUT2,
                apower: TriangleInequalityAlgorithm::DEFAULT_APOWER,
            },
            Self::StripeAverage {
                bailout2: StripeAverageAlgorithm::DEFAULT_BAILOUT2,
                skip_iteration: StripeAverageAlgorithm::DEFAULT_SKIP_ITERATION,
                stripe_density: StripeAverageAlgorithm::DEFAULT_STRIPE_DENSITY,
            },
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Colour { .. } => "Colour",
            Self::OrbitTrap { .. } => "Orbit Trap",
            Self::Shading3D { .. } => "Shading3D",
            Self::TriangleInequality { .. } => "Triangle Inequality",
            Self::StripeAverage { .. } => "Stripe Average",
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LayerMappingKind {
    Blend,
    Shade,
}

/// Specifies the range of the fractal set a layer is applied to.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
impl crate::ui::Dropdown<LayerRange> for LayerRange {
    fn get_variants() -> Vec<LayerRange> {
        vec![LayerRange::InSet, LayerRange::OutSet, LayerRange::Both]
    }

    fn get_text(&self) -> &str {
        match self {
            LayerRange::InSet => "In set",
            LayerRange::OutSet => "Out set",
            LayerRange::Both => "Both",
        }
    }
}
