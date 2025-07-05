/// Layering algorithms that can be run while analysing a point/pixel.
use crate::{rendering::orbit_trap::*, types::*};
use std::f64::consts::PI;

/// Operations algorithms should perform while analysing a point/pixel.
pub trait LayerAlgorithm {
    /// What needs to happen before iterations start for a given pixel.
    fn before(&mut self, max_iterations: u32, bailout2: f64);

    /// What needs to happen each iteration, before the next z is calculated
    /// for double precision.
    fn during_double(&mut self, z: Complex, i: u32);
    /// What needs to happen each iteration, before the next z is calculated
    /// for arbitrary precision.
    fn during_big(&mut self, z: &BigComplex, i: u32);

    /// What needs to happen if the point is outside the set
    /// for double precision.
    fn out_set_double(&mut self, z: Complex, i: u32);
    /// What needs to happen if the point is outside the set
    /// for arbitrary precision.
    fn out_set_big(&mut self, z: &BigComplex, i: u32);

    /// What needs to happen if the point is inside the set
    /// for double precision.
    fn in_set_double(&mut self, z: Complex);
    /// What needs to happen if the point is inside the set
    /// for arbitrary precision.
    fn in_set_big(&mut self, z: &BigComplex);

    /// Returns the specific output value for this algorithm.
    fn get_output(&self) -> f64;
}

// Done instead of box(dyn ...) for better cache locality, memory utilisation, and performance.
/// Delegates the given `self.fn(*args)` to the correct [`LayerImplementation`] variant.
macro_rules! delegate_layer_implementation {
    ($self:ident.$method:ident($($arg:expr),*)) => {
        match $self {
            LayerImplementation::Colour(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::OrbitTrap(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::Shading3D(algorithm) => algorithm.$method($($arg),*),
            // LayerImplementation::TriangleInequality(algorithm) => algorithm.$method($($arg),*),
            #[allow(unreachable_patterns)]
            _ => unreachable!("New variant added but not handled")
        }
    }
}

/// An implementation of a layering algorithm.
#[repr(u8)]
pub enum LayerImplementation {
    Colour(ColourAlgorithm),
    OrbitTrap(OrbitTrapAlgorithm),
    Shading3D(Shading3DAlgorithm),
    TriangleInequality(TriangleInequalityAlgorithm),
}
impl LayerAlgorithm for LayerImplementation {
    fn before(&mut self, max_iterations: u32, bailout2: f64) {
        delegate_layer_implementation!(self.before(max_iterations, bailout2))
    }

    fn during_double(&mut self, z: Complex, i: u32) {
        delegate_layer_implementation!(self.during_double(z, i))
    }
    fn during_big(&mut self, z: &BigComplex, i: u32) {
        delegate_layer_implementation!(self.during_big(z, i))
    }

    fn out_set_double(&mut self, z: Complex, i: u32) {
        delegate_layer_implementation!(self.out_set_double(z, i))
    }
    fn out_set_big(&mut self, z: &BigComplex, i: u32) {
        delegate_layer_implementation!(self.out_set_big(z, i))
    }

    fn in_set_double(&mut self, z: Complex) {
        delegate_layer_implementation!(self.in_set_double(z))
    }
    fn in_set_big(&mut self, z: &BigComplex) {
        delegate_layer_implementation!(self.in_set_big(z))
    }

    fn get_output(&self) -> f64 {
        delegate_layer_implementation!(self.get_output())
    }
}

/// Simple smooth colouring algorithm tracking the number of iterations a point takes to diverge, and returning
/// the smoothed iteration value.
pub struct ColourAlgorithm {
    output: f64,
}
impl ColourAlgorithm {
    pub fn new() -> Self {
        Self { output: 0.0 }
    }

    fn out_set(&mut self, abs2_z: f64, i: u32) {
        // Perform smooth iteration algorithm
        let log_zmod = f64::log2(abs2_z) / 2.0;
        let nu = f64::log2(log_zmod);
        let smooth_iteration = i as f64 + 1.0 - nu;
        self.output = smooth_iteration;
    }
}
impl LayerAlgorithm for ColourAlgorithm {
    fn before(&mut self, _max_iterations: u32, _bailout2: f64) {}

    fn during_double(&mut self, _z: Complex, _i: u32) {}
    fn during_big(&mut self, _z: &BigComplex, _i: u32) {}

    fn out_set_double(&mut self, z: Complex, i: u32) {
        self.out_set(z.abs_squared(), i);
    }
    fn out_set_big(&mut self, z: &BigComplex, i: u32) {
        self.out_set(z.abs_squared(), i);
    }

    fn in_set_double(&mut self, _z: Complex) {
        self.output = 0.0;
    }
    fn in_set_big(&mut self, _z: &BigComplex) {
        self.output = 0.0;
    }

    fn get_output(&self) -> f64 {
        self.output
    }
}

/// Orbit trap algorithm looking at the minimum distance between an orbit and an orbit trap,
/// calculating a trapped index to be used in the palette.
pub struct OrbitTrapAlgorithm {
    output: f64,
    min_distance2: f64,
    divisor: f64,
    trap: OrbitTrapType,
    /// 'vector' of closest point to the trap
    closest_to_trap: Complex,
    closest_to_trap_big: BigComplex,
}
impl OrbitTrapAlgorithm {
    pub fn new(trap: OrbitTrapType) -> Self {
        Self {
            output: 0.0,
            min_distance2: f64::INFINITY,
            divisor: 0.0,
            trap,
            closest_to_trap: Complex::new(0.0, 0.0),
            closest_to_trap_big: BigComplex::from_f64s(0.0, 0.0),
        }
    }

    fn generate_output_double(&self) -> f64 {
        let output = match self.trap.get_analysis() {
            OrbitTrapAnalysis::Distance => self.min_distance2.sqrt(),
            OrbitTrapAnalysis::Real => self.closest_to_trap.real_f64().abs(),
            OrbitTrapAnalysis::Imaginary => self.closest_to_trap.im_f64().abs(),
            OrbitTrapAnalysis::Angle => PI + self.closest_to_trap.arg(),
        } / self.divisor;
        output
    }
    fn generate_output_big(&self) -> f64 {
        let output = match self.trap.get_analysis() {
            OrbitTrapAnalysis::Distance => self.min_distance2.sqrt(),
            OrbitTrapAnalysis::Real => self.closest_to_trap_big.real_f64().abs(),
            OrbitTrapAnalysis::Imaginary => self.closest_to_trap_big.im_f64().abs(),
            OrbitTrapAnalysis::Angle => PI + self.closest_to_trap_big.arg(),
        } / self.divisor;
        output
    }
}
impl LayerAlgorithm for OrbitTrapAlgorithm {
    fn before(&mut self, max_iterations: u32, bailout2: f64) {
        self.min_distance2 = self.trap.greatest_distance2(bailout2);
        self.divisor = self.min_distance2.sqrt() / max_iterations as f64;
    }

    fn during_double(&mut self, z: Complex, _i: u32) {
        let z_trap_distance2 = self.trap.distance2_double(z);
        if z_trap_distance2 < self.min_distance2 {
            self.min_distance2 = z_trap_distance2;
            self.closest_to_trap = self.trap.vector_double(z);
        }
    }
    fn during_big(&mut self, z: &BigComplex, _i: u32) {
        let z_trap_distance2 = self.trap.distance2_big(z);
        if z_trap_distance2 < self.min_distance2 {
            self.min_distance2 = z_trap_distance2;
            self.closest_to_trap_big = self.trap.vector_big(z);
        }
    }

    fn out_set_double(&mut self, _z: Complex, _i: u32) {
        self.output = self.generate_output_double();
    }
    fn out_set_big(&mut self, _z: &BigComplex, _i: u32) {
        self.output = self.generate_output_big();
    }

    fn in_set_double(&mut self, _z: Complex) {
        self.output = self.generate_output_double();
    }
    fn in_set_big(&mut self, _z: &BigComplex) {
        self.output = self.generate_output_big();
    }

    fn get_output(&self) -> f64 {
        self.output
    }
}

/// 3d algorithm to shade the set to give height.
/// Theory from: https://www.math.univ-toulouse.fr/~cheritat/wiki-draw/index.php/Mandelbrot_set#Normal_map_effect,
/// calculating a t value that represents darkness/brightness.
pub struct Shading3DAlgorithm {
    output: f64,
    v: Complex,
    v_big: BigComplex,
    der: Complex,
    der_big: BigComplex,
    dc: Complex,
    dc_big: BigComplex,
}
impl Shading3DAlgorithm {
    const H2: f64 = 1.5;
    const ANGLE: f64 = -45.0;

    pub fn new() -> Self {
        Self {
            output: 0.0,
            v: Complex::new(
                f64::cos(Self::ANGLE * (PI / 180.)),
                f64::sin(Self::ANGLE * (PI / 180.)),
            ),
            v_big: BigComplex::from_f64s(
                f64::cos(Self::ANGLE * (PI / 180.)),
                f64::sin(Self::ANGLE * (PI / 180.)),
            ),
            der: Complex::new(1., 0.),
            der_big: BigComplex::from_f64s(1., 0.),
            dc: Complex::new(1., 0.),
            dc_big: BigComplex::from_f64s(1., 0.),
        }
    }

    fn generate_output_double(&self, z: Complex) -> f64 {
        let mut u = z / self.der;
        u = &u / f64::sqrt(u.abs_squared());
        let t = u.real * self.v.real + u.im * self.v.im;
        let mut t = t + Self::H2;
        t = t / (1. + Self::H2);
        t = f64::max(0.1, t);
        t
    }

    fn generate_output_big(&self, z: &BigComplex) -> f64 {
        let mut u = z / &self.der_big;
        u = &u / f64::sqrt(u.abs_squared());
        let t = u.real_f64() * self.v_big.real_f64() + u.im_f64() * self.v_big.im_f64();
        let mut t = t + Self::H2;
        t = t / (1. + Self::H2);
        t = f64::max(0.1, t);
        t
    }
}
impl LayerAlgorithm for Shading3DAlgorithm {
    fn before(&mut self, _max_iterations: u32, _bailout2: f64) {}

    fn during_double(&mut self, z: Complex, _i: u32) {
        self.der = self.der * (z * 2.) + self.dc;
    }
    fn during_big(&mut self, z: &BigComplex, _i: u32) {
        self.der_big = &self.der_big * (z * 2.) + &self.dc_big;
    }

    fn out_set_double(&mut self, z: Complex, _i: u32) {
        self.output = self.generate_output_double(z);
    }
    fn out_set_big(&mut self, z: &BigComplex, _i: u32) {
        self.output = self.generate_output_big(z);
    }

    fn in_set_double(&mut self, z: Complex) {
        self.output = self.generate_output_double(z);
    }
    fn in_set_big(&mut self, z: &BigComplex) {
        self.output = self.generate_output_big(z);
    }

    fn get_output(&self) -> f64 {
        self.output
    }
}

/// Triangle inequality algorithm
pub struct TriangleInequalityAlgorithm {}
impl TriangleInequalityAlgorithm {
    pub fn new() -> Self {
        Self {}
    }
}
// impl LayerAlgorithm for TriangleInequalityAlgorithm {}
