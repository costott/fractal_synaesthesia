/// Orbit traps: Shapes in the complex plane.
use dashu_float::FBig;
use std::f64::consts::PI;

use crate::types::*;

/// Determines what metric will be used to analyse a point with the orbit trap.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum OrbitTrapAnalysis {
    Distance,
    Real,
    Imaginary,
    Angle,
}
impl crate::ui::Dropdown<OrbitTrapAnalysis> for OrbitTrapAnalysis {
    fn get_variants() -> Vec<OrbitTrapAnalysis> {
        vec![
            OrbitTrapAnalysis::Distance,
            OrbitTrapAnalysis::Real,
            OrbitTrapAnalysis::Imaginary,
            OrbitTrapAnalysis::Angle,
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            OrbitTrapAnalysis::Distance => "Distance",
            OrbitTrapAnalysis::Real => "Real",
            OrbitTrapAnalysis::Imaginary => "Imaginary",
            OrbitTrapAnalysis::Angle => "Angle",
        }
    }
}

/// Operations every orbit trap should perform.
pub trait OrbitTrap {
    /// Returns a vector of the given (double) complex number `z` to the trap.
    fn vector_double(&self, z: Complex) -> Complex;
    /// Returns a vector of the given (arbitrary) complex number `z` to the trap.
    fn vector_big(&self, z: &BigComplex) -> BigComplex;

    /// Returns the squared distance between the given (double) complex number `z` to the trap.
    fn distance2_double(&self, z: Complex) -> f64;
    /// Returns the squared distance between the given (arbitrary) complex number `z` to the trap.
    fn distance2_big(&self, z: &BigComplex) -> f64;

    /// Returns the greatest possible squared distance of a point to the trap.
    fn greatest_distance2(&self, bailout2: f64) -> f64;

    fn get_analysis(&self) -> OrbitTrapAnalysis;
    fn get_analysis_mut(&mut self) -> &mut OrbitTrapAnalysis;
    fn set_analysis(&mut self, new: OrbitTrapAnalysis);

    /// Returns the real part of the center of the trap.
    fn get_center_re(&self) -> f64;
    /// Sets the real part of the center of the trap.
    fn set_center_re(&mut self, new: f64);
    /// Returns the imaginary part of the center of the trap.
    fn get_center_im(&self) -> f64;
    /// Sets the imaginary part of the center of the trap.
    fn set_center_im(&mut self, new: f64);
}

// Done instead of box(dyn ...) for better cache locality, memory utilisation, and performance.
/// Delegates the given `self.fn(*args)` to the correct [`OrbitTrapType`] variant.
macro_rules! delegate_orbit_trap_type {
    ($self:ident.$method:ident($($arg:expr),*)) => {
        match $self {
            OrbitTrapType::Point(trap) => trap.$method($($arg),*),
            OrbitTrapType::Cross(trap) => trap.$method($($arg),*),
            OrbitTrapType::Circle(trap) => trap.$method($($arg),*),
            #[allow(unreachable_patterns)]
            _ => unreachable!("New variant added but not handled")
        }
    }
}

/// A wrapper around the different orbit trap types for disatching.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum OrbitTrapType {
    Point(OrbitTrapPoint),
    Cross(OrbitTrapCross),
    Circle(OrbitTrapCircle),
}
impl OrbitTrap for OrbitTrapType {
    fn vector_double(&self, z: Complex) -> Complex {
        delegate_orbit_trap_type!(self.vector_double(z))
    }
    fn vector_big(&self, z: &BigComplex) -> BigComplex {
        delegate_orbit_trap_type!(self.vector_big(z))
    }

    fn distance2_double(&self, z: Complex) -> f64 {
        delegate_orbit_trap_type!(self.distance2_double(z))
    }
    fn distance2_big(&self, z: &BigComplex) -> f64 {
        delegate_orbit_trap_type!(self.distance2_big(z))
    }

    fn greatest_distance2(&self, bailout2: f64) -> f64 {
        delegate_orbit_trap_type!(self.greatest_distance2(bailout2))
    }

    fn get_analysis(&self) -> OrbitTrapAnalysis {
        delegate_orbit_trap_type!(self.get_analysis())
    }
    fn get_analysis_mut(&mut self) -> &mut OrbitTrapAnalysis {
        delegate_orbit_trap_type!(self.get_analysis_mut())
    }
    fn set_analysis(&mut self, new: OrbitTrapAnalysis) {
        delegate_orbit_trap_type!(self.set_analysis(new))
    }

    fn get_center_re(&self) -> f64 {
        delegate_orbit_trap_type!(self.get_center_re())
    }
    fn set_center_re(&mut self, new: f64) {
        delegate_orbit_trap_type!(self.set_center_re(new))
    }

    fn get_center_im(&self) -> f64 {
        delegate_orbit_trap_type!(self.get_center_im())
    }
    fn set_center_im(&mut self, new: f64) {
        delegate_orbit_trap_type!(self.set_center_im(new))
    }
}
impl Default for OrbitTrapType {
    fn default() -> Self {
        Self::Point(OrbitTrapPoint::default())
    }
}
impl crate::ui::Dropdown<OrbitTrapType> for OrbitTrapType {
    fn get_variants() -> Vec<OrbitTrapType> {
        vec![
            OrbitTrapType::Point(OrbitTrapPoint::default()),
            OrbitTrapType::Cross(OrbitTrapCross::default()),
            OrbitTrapType::Circle(OrbitTrapCircle::default()),
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            OrbitTrapType::Point(_) => "Point",
            OrbitTrapType::Cross(_) => "Cross",
            OrbitTrapType::Circle(_) => "Circle",
        }
    }
}

/// Macro for the adding the editing orbit trap code as it's identical for each type.
macro_rules! edit_orbit_trap {
    () => {
        fn get_analysis(&self) -> OrbitTrapAnalysis {
            self.analysis
        }
        fn get_analysis_mut(&mut self) -> &mut OrbitTrapAnalysis {
            &mut self.analysis
        }
        fn set_analysis(&mut self, new: OrbitTrapAnalysis) {
            self.analysis = new;
        }

        fn get_center_re(&self) -> f64 {
            self.center.real
        }
        fn set_center_re(&mut self, new: f64) {
            self.center.real = new;
            self.big_center.real = FBig::try_from(new).unwrap();
        }

        fn get_center_im(&self) -> f64 {
            self.center.im
        }
        fn set_center_im(&mut self, new: f64) {
            self.center.im = new;
            self.big_center.im = FBig::try_from(new).unwrap();
        }
    };
}

/// An orbit trap which shape is a single point in the complex plane.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OrbitTrapPoint {
    pub center: Complex,
    big_center: BigComplex,
    pub analysis: OrbitTrapAnalysis,
}
impl OrbitTrapPoint {
    pub fn new(point: (f64, f64), analysis: OrbitTrapAnalysis) -> OrbitTrapPoint {
        OrbitTrapPoint {
            center: Complex::new(point.0, point.1),
            big_center: BigComplex::from_f64s(point.0, point.1),
            analysis,
        }
    }
}
impl Default for OrbitTrapPoint {
    fn default() -> Self {
        OrbitTrapPoint::new((0., 0.), OrbitTrapAnalysis::Distance)
    }
}
impl OrbitTrap for OrbitTrapPoint {
    fn vector_double(&self, z: Complex) -> Complex {
        z - self.center
    }
    fn vector_big(&self, z: &BigComplex) -> BigComplex {
        z - &self.big_center
    }

    fn distance2_double(&self, z: Complex) -> f64 {
        (z - self.center).abs_squared()
    }
    fn distance2_big(&self, z: &BigComplex) -> f64 {
        (z - &self.big_center).abs_squared()
    }

    fn greatest_distance2(&self, bailout2: f64) -> f64 {
        let big_rad = bailout2.sqrt();
        match self.analysis {
            OrbitTrapAnalysis::Distance => (big_rad + self.center.abs_squared().sqrt()).powi(2),
            OrbitTrapAnalysis::Real => (big_rad + self.center.real.abs()).powi(2),
            OrbitTrapAnalysis::Imaginary => (big_rad + self.center.im.abs()).powi(2),
            OrbitTrapAnalysis::Angle => (2. * PI).powi(2),
        }
    }

    edit_orbit_trap!();
}
impl PartialEq for OrbitTrapPoint {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.analysis == other.analysis
    }
}
impl Eq for OrbitTrapPoint {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OrbitTrapCross {
    pub center: Complex,
    big_center: BigComplex,
    pub arm_length: f64,
    pub analysis: OrbitTrapAnalysis,
}
impl OrbitTrapCross {
    pub fn new(centre: (f64, f64), arm_length: f64, analysis: OrbitTrapAnalysis) -> OrbitTrapCross {
        OrbitTrapCross {
            center: Complex::new(centre.0, centre.1),
            big_center: BigComplex::from_f64s(centre.0, centre.1),
            arm_length,
            analysis,
        }
    }
}
impl Default for OrbitTrapCross {
    fn default() -> Self {
        OrbitTrapCross::new((0., 0.), 1., OrbitTrapAnalysis::Distance)
    }
}
impl OrbitTrap for OrbitTrapCross {
    fn vector_double(&self, z: Complex) -> Complex {
        let vector;
        let x_dist = z.real - self.center.real;
        let y_dist = z.im - self.center.im;
        if self.center.im - self.arm_length <= z.im
            && z.im <= self.center.im + self.arm_length
            && self.center.real - self.arm_length <= z.real
            && z.real <= self.center.real + self.arm_length
        {
            vector = if x_dist.abs() <= y_dist.abs() {
                Complex::new(x_dist, 0.)
            } else {
                Complex::new(0., y_dist)
            }
        } else if x_dist.abs() < y_dist.abs() {
            vector = z - Complex::new(
                self.center.real,
                self.center.im + y_dist.signum() * self.arm_length,
            )
        } else {
            vector = z - Complex::new(
                self.center.real + x_dist.signum() * self.arm_length,
                self.center.im,
            )
        }

        vector
    }
    fn vector_big(&self, _z: &BigComplex) -> BigComplex {
        BigComplex::from_f64s(0.0, 0.0)
    }

    fn distance2_double(&self, z: Complex) -> f64 {
        self.vector_double(z).abs_squared()
    }
    fn distance2_big(&self, z: &BigComplex) -> f64 {
        self.distance2_double(z.as_complex())
    }

    /// returns the maximum possible distance
    /// a complex number can be from the trap
    fn greatest_distance2(&self, bailout2: f64) -> f64 {
        match self.analysis {
            OrbitTrapAnalysis::Distance => {
                (bailout2.sqrt() + self.center.abs_squared().sqrt()).powi(2)
            }
            OrbitTrapAnalysis::Real => {
                (bailout2.sqrt() + self.center.real.abs() - self.arm_length).powi(2)
            }
            OrbitTrapAnalysis::Imaginary => {
                (bailout2.sqrt() + self.center.im.abs() - self.arm_length).powi(2)
            }
            OrbitTrapAnalysis::Angle => (2. * PI).powi(2),
        }
    }

    edit_orbit_trap!();
}
impl PartialEq for OrbitTrapCross {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && self.analysis == other.analysis
            && self.arm_length == other.arm_length
    }
}
impl Eq for OrbitTrapCross {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OrbitTrapCircle {
    pub center: Complex,
    big_center: BigComplex,
    pub radius: f64,
    pub analysis: OrbitTrapAnalysis,
}
impl OrbitTrapCircle {
    pub fn new(centre: (f64, f64), radius: f64, analysis: OrbitTrapAnalysis) -> OrbitTrapCircle {
        OrbitTrapCircle {
            center: Complex::new(centre.0, centre.1),
            big_center: BigComplex::from_f64s(centre.0, centre.1),
            radius,
            analysis,
        }
    }
}
impl Default for OrbitTrapCircle {
    fn default() -> Self {
        OrbitTrapCircle::new((0., 0.), 1., OrbitTrapAnalysis::Distance)
    }
}
impl OrbitTrap for OrbitTrapCircle {
    fn vector_double(&self, z: Complex) -> Complex {
        let dist_rem = self.distance2_double(z).sqrt() % self.radius;
        ((z - self.center) / (z - self.center).abs_squared().sqrt()) * dist_rem
    }
    fn vector_big(&self, z: &BigComplex) -> BigComplex {
        let dist_rem = self.distance2_big(z).sqrt() % self.radius;
        ((z - &self.big_center) / (z - &self.big_center).abs_squared().sqrt()) * dist_rem
    }

    fn distance2_double(&self, z: Complex) -> f64 {
        ((z - self.center).abs_squared().sqrt() - self.radius).powi(2)
    }
    fn distance2_big(&self, z: &BigComplex) -> f64 {
        ((z - &self.big_center).abs_squared().sqrt() - self.radius).powi(2)
    }

    fn greatest_distance2(&self, bailout2: f64) -> f64 {
        let big_rad = bailout2.sqrt();

        match self.analysis {
            OrbitTrapAnalysis::Distance => f64::max(
                big_rad - (self.radius - self.center.abs_squared().sqrt()),
                self.radius,
            )
            .powi(2),
            OrbitTrapAnalysis::Real => big_rad + self.center.real - self.radius,
            OrbitTrapAnalysis::Imaginary => big_rad + self.center.im - self.radius,
            OrbitTrapAnalysis::Angle => (2. * PI).powi(2),
        }
    }

    edit_orbit_trap!();
}
impl PartialEq for OrbitTrapCircle {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && self.analysis == other.analysis
            && self.radius == other.radius
    }
}
impl Eq for OrbitTrapCircle {}

#[cfg(test)]
mod tests {
    use super::*;
}
