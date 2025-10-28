/// Layering algorithms that can be run while analysing a point/pixel.
use crate::{rendering::orbit_trap::*, types::*};
use std::f64::consts::PI;

/// Operations algorithms should perform while analysing a point/pixel.
pub trait LayerAlgorithm {
    /// Returns the squared bailout radius for this layer
    fn get_bailout2(&self) -> f64;

    /// What needs to happen before iterations start for a given pixel.
    fn before(&mut self, max_iterations: u32);

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
    /// Technically an iteration value betweeen [0, max_iterations)
    fn get_output(&self) -> f64;

    /// Returns whether the algorithm analysed that the point was in the fractal set or not.
    fn get_in_set(&self) -> bool;
}

// Done instead of box(dyn ...) for better cache locality, memory utilisation, and performance.
/// Delegates the given `self.fn(*args)` to the correct [`LayerImplementation`] variant.
macro_rules! delegate_layer_implementation {
    ($self:ident.$method:ident($($arg:expr),*)) => {
        match $self {
            LayerImplementation::Colour(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::OrbitTrap(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::Shading3D(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::TriangleInequality(algorithm) => algorithm.$method($($arg),*),
            LayerImplementation::StripeAverage(algorithm) => algorithm.$method($($arg),*),
            #[allow(unreachable_patterns)]
            _ => unreachable!("New variant added but not handled")
        }
    }
}

macro_rules! delegate_in_set {
    ($self:ident, $bool:expr) => {
        match $self {
            LayerImplementation::Colour(algorithm) => algorithm.in_set = $bool,
            LayerImplementation::OrbitTrap(algorithm) => algorithm.in_set = $bool,
            LayerImplementation::Shading3D(algorithm) => algorithm.in_set = $bool,
            LayerImplementation::TriangleInequality(algorithm) => algorithm.in_set = $bool,
            LayerImplementation::StripeAverage(algorithm) => algorithm.in_set = $bool,
            #[allow(unreachable_patterns)]
            _ => unreachable!("New variant added but not handled"),
        }
    };
}

/// An implementation of a layering algorithm.
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum LayerImplementation {
    Colour(ColourAlgorithm),
    OrbitTrap(OrbitTrapAlgorithm),
    Shading3D(Shading3DAlgorithm),
    TriangleInequality(TriangleInequalityAlgorithm),
    StripeAverage(StripeAverageAlgorithm),
}
impl LayerAlgorithm for LayerImplementation {
    fn get_bailout2(&self) -> f64 {
        delegate_layer_implementation!(self.get_bailout2())
    }

    fn before(&mut self, max_iterations: u32) {
        delegate_layer_implementation!(self.before(max_iterations))
    }

    fn during_double(&mut self, z: Complex, i: u32) {
        delegate_layer_implementation!(self.during_double(z, i))
    }
    fn during_big(&mut self, z: &BigComplex, i: u32) {
        delegate_layer_implementation!(self.during_big(z, i))
    }

    fn out_set_double(&mut self, z: Complex, i: u32) {
        delegate_layer_implementation!(self.out_set_double(z, i));
        delegate_in_set!(self, false);
    }
    fn out_set_big(&mut self, z: &BigComplex, i: u32) {
        delegate_layer_implementation!(self.out_set_big(z, i));
        delegate_in_set!(self, false);
    }

    fn in_set_double(&mut self, z: Complex) {
        delegate_layer_implementation!(self.in_set_double(z));
        delegate_in_set!(self, true);
    }
    fn in_set_big(&mut self, z: &BigComplex) {
        delegate_layer_implementation!(self.in_set_big(z));
        delegate_in_set!(self, true);
    }

    fn get_output(&self) -> f64 {
        delegate_layer_implementation!(self.get_output())
    }

    fn get_in_set(&self) -> bool {
        delegate_layer_implementation!(self.get_in_set())
    }
}

/// Simple smooth colouring algorithm tracking the number of iterations a point takes to diverge, and returning
/// the smoothed iteration value.
#[derive(Clone, Copy, Debug)]
pub struct ColourAlgorithm {
    bailout2: f64,
    output: f64,
    in_set: bool,
}
impl ColourAlgorithm {
    fn factory(bailout2: f64) -> Self {
        Self {
            bailout2,
            output: 0.0,
            in_set: false,
        }
    }

    pub fn new() -> Self {
        Self {
            output: 0.0,
            ..Default::default()
        }
    }

    fn out_set(&mut self, abs2_z: f64, i: u32) {
        // Perform smooth iteration algorithm
        let log_zmod = f64::log2(abs2_z) * 0.5;
        let nu = f64::log2(log_zmod);
        let smooth_iteration = i as f64 + 1.0 - nu;
        self.output = smooth_iteration;
    }
}
impl LayerAlgorithm for ColourAlgorithm {
    fn get_bailout2(&self) -> f64 {
        self.bailout2
    }

    fn before(&mut self, _max_iterations: u32) {}

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

    fn get_in_set(&self) -> bool {
        self.in_set
    }
}
impl Default for ColourAlgorithm {
    fn default() -> Self {
        Self::factory(10.)
    }
}

/// Orbit trap algorithm looking at the minimum distance between an orbit and an orbit trap,
/// calculating a trapped index to be used in the palette.
#[derive(Clone, Debug)]
pub struct OrbitTrapAlgorithm {
    bailout2: f64,
    min_distance2: f64,
    divisor: f64,
    trap: OrbitTrapType,
    /// 'vector' of closest point to the trap
    closest_to_trap: Complex,
    closest_to_trap_big: BigComplex,
    output: f64,
    in_set: bool,
}
impl OrbitTrapAlgorithm {
    const DEFAULT_BAILOUT2: f64 = 4.5;

    fn factory(trap: OrbitTrapType, bailout2: f64) -> Self {
        Self {
            bailout2,
            min_distance2: f64::INFINITY,
            divisor: 0.0,
            trap,
            closest_to_trap: Complex::new(0.0, 0.0),
            closest_to_trap_big: BigComplex::from_f64s(0.0, 0.0),
            output: 0.0,
            in_set: false,
        }
    }

    pub fn new(trap: OrbitTrapType) -> Self {
        Self::factory(trap, Self::DEFAULT_BAILOUT2)
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
    fn get_bailout2(&self) -> f64 {
        self.bailout2
    }

    fn before(&mut self, max_iterations: u32) {
        self.min_distance2 = self.trap.greatest_distance2(self.bailout2);
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
        self.in_set = false;
    }
    fn out_set_big(&mut self, _z: &BigComplex, _i: u32) {
        self.output = self.generate_output_big();
        self.in_set = false;
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

    fn get_in_set(&self) -> bool {
        self.in_set
    }
}
impl Default for OrbitTrapAlgorithm {
    fn default() -> Self {
        Self::factory(
            OrbitTrapType::Point(OrbitTrapPoint::default()),
            Self::DEFAULT_BAILOUT2,
        )
    }
}

/// 3d algorithm to shade the set to give height.
/// Theory from: https://www.math.univ-toulouse.fr/~cheritat/wiki-draw/index.php/Mandelbrot_set#Normal_map_effect,
/// calculating a t value that represents darkness/brightness.
#[derive(Clone, Debug)]
pub struct Shading3DAlgorithm {
    bailout2: f64,
    h2: f64,
    v: Complex,
    v_big: BigComplex,
    der: Complex,
    der_big: BigComplex,
    dc: Complex,
    dc_big: BigComplex,
    output: f64,
    in_set: bool,
}
impl Shading3DAlgorithm {
    pub const DEFAULT_H2: f64 = 1.5;
    pub const DEFAULT_ANGLE: f64 = 45.0;
    const DEFAULT_BAILOUT2: f64 = 1e8;

    fn factory(h2: f64, angle: f64, bailout2: f64) -> Self {
        Self {
            bailout2,
            h2,
            v: Complex::new(f64::cos(angle * (PI / 180.)), f64::sin(angle * (PI / 180.))),
            v_big: BigComplex::from_f64s(
                f64::cos(angle * (PI / 180.)),
                f64::sin(angle * (PI / 180.)),
            ),
            der: Complex::new(1., 0.),
            der_big: BigComplex::from_f64s(1., 0.),
            dc: Complex::new(1., 0.),
            dc_big: BigComplex::from_f64s(1., 0.),
            output: 0.0,
            in_set: false,
        }
    }

    pub fn new(h2: f64, angle: f64) -> Self {
        Self::factory(h2, angle, Self::DEFAULT_BAILOUT2)
    }

    fn generate_output_double(&self, z: Complex) -> f64 {
        let mut u = z / self.der;
        u = &u / f64::sqrt(u.abs_squared());
        let t = u.real * self.v.real + u.im * self.v.im;
        let mut t = t + self.h2;
        t = t / (1. + self.h2);
        t = f64::max(0.0, t);
        t
    }

    fn generate_output_big(&self, z: &BigComplex) -> f64 {
        let mut u = z / &self.der_big;
        u = &u / f64::sqrt(u.abs_squared());
        let t = u.real_f64() * self.v_big.real_f64() + u.im_f64() * self.v_big.im_f64();
        let mut t = t + self.h2;
        t = t / (1. + self.h2);
        t = f64::max(0.0, t);
        t
    }
}
impl LayerAlgorithm for Shading3DAlgorithm {
    fn get_bailout2(&self) -> f64 {
        self.bailout2
    }

    fn before(&mut self, _max_iterations: u32) {}

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

    fn get_in_set(&self) -> bool {
        self.in_set
    }
}
impl Default for Shading3DAlgorithm {
    fn default() -> Self {
        Self::factory(
            Self::DEFAULT_H2,
            Self::DEFAULT_ANGLE,
            Self::DEFAULT_BAILOUT2,
        )
    }
}

/// Triangle inequality algorithm https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/triangle_ineq
#[derive(Clone, Debug)]
pub struct TriangleInequalityAlgorithm {
    bailout2: f64,
    max_iter: u32,
    sum: f64,
    sum2: f64,
    ac: f64,
    lp: f64,
    ipower: f64,
    first_double: Option<Complex>,
    first_big: Option<BigComplex>,
    output: f64,
    in_set: bool,
}
impl TriangleInequalityAlgorithm {
    const DEFAULT_BAILOUT2: f64 = 1e40;
    /// Average power: skews values averaged by raising them to this power.
    pub const DEFAULT_APOWER: f64 = 1.0;

    fn factory(apower: f64, bailout2: f64) -> Self {
        Self {
            bailout2,
            max_iter: Default::default(),
            sum: 0.0,
            sum2: 0.0,
            ac: Default::default(),
            lp: Default::default(),
            ipower: 1.0 / apower,
            first_double: None,
            first_big: None,
            output: 0.0,
            in_set: false,
        }
    }

    pub fn new(apower: f64) -> Self {
        Self::factory(apower, Self::DEFAULT_BAILOUT2)
    }

    fn get_output_double(&mut self, z: Complex, i: u32) -> f64 {
        self.sum = self.sum / i as f64;
        self.sum2 = self.sum2 / (i - 1) as f64;
        let f = 1.0 * self.lp - 1.0 * f64::log2(f64::log2(z.abs_squared()) * 0.5);
        let i_frac = self.sum2 + (self.sum - self.sum2) * (f + 1.0);
        i_frac * self.max_iter as f64
    }

    fn get_output_big(&mut self, z: &BigComplex, i: u32) -> f64 {
        self.sum = self.sum / i as f64;
        self.sum2 = self.sum2 / (i - 1) as f64;
        let f = 1.0 * self.lp - 1.0 * f64::log2(f64::log2(z.abs_squared()) * 0.5);
        let i_frac = self.sum2 + (self.sum - self.sum2) * (f + 1.0);
        i_frac * self.max_iter as f64
    }
}
impl LayerAlgorithm for TriangleInequalityAlgorithm {
    fn get_bailout2(&self) -> f64 {
        self.bailout2
    }

    fn before(&mut self, max_iterations: u32) {
        self.max_iter = max_iterations;
        // log(log(bailout) / 2)) = log(log(bailout^2) / 4)
        self.lp = f64::log2(f64::log2(self.bailout2) * 0.25)
    }

    fn during_double(&mut self, z: Complex, _i: u32) {
        self.sum2 = self.sum;
        if let Some(z0) = self.first_double {
            let az2 = (z - z0).abs_squared().sqrt();
            let lowbound = (az2 - self.ac).abs();
            let numerator = z.abs_squared().sqrt() - lowbound;
            let denominator = az2 + self.ac - lowbound;

            if denominator.abs() >= f64::EPSILON {
                self.sum += (numerator / denominator).powf(self.ipower);
            }
        } else {
            self.ac = z.abs_squared().sqrt();
            self.first_double = Some(z)
        }
    }

    fn during_big(&mut self, z: &BigComplex, _i: u32) {
        self.sum2 = self.sum;
        if let Some(z0) = &self.first_big {
            let az2 = (z - &z0).abs_squared().sqrt();
            let lowbound = (az2 - self.ac).abs();
            self.sum += ((z.abs_squared().sqrt() - lowbound) / (az2 + self.ac - lowbound))
                .powf(self.ipower);
        } else {
            self.ac = z.abs_squared().sqrt();
            self.first_big = Some(z.clone())
        }
    }

    fn out_set_double(&mut self, z: Complex, i: u32) {
        self.output = self.get_output_double(z, i);
    }

    fn out_set_big(&mut self, z: &BigComplex, i: u32) {
        self.output = self.get_output_big(z, i);
    }

    fn in_set_double(&mut self, z: Complex) {
        self.output = self.get_output_double(z, self.max_iter - 1);
    }

    fn in_set_big(&mut self, z: &BigComplex) {
        self.output = self.get_output_big(z, self.max_iter - 1);
    }

    fn get_output(&self) -> f64 {
        self.output
    }

    fn get_in_set(&self) -> bool {
        self.in_set
    }
}
impl Default for TriangleInequalityAlgorithm {
    fn default() -> Self {
        Self::factory(Self::DEFAULT_APOWER, Self::DEFAULT_BAILOUT2)
    }
}

/// Stripe average algorithm https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/stripeAC
/// with linear interpolation and skipped orbits
#[derive(Clone, Debug)]
pub struct StripeAverageAlgorithm {
    bailout2: f64,
    /// Exclude all iterations less than or equal to this number (k)
    skip_iteration: u32,
    stripe_density: f64,
    // t_{k+1} + t_{k+2} + ... + t_{n}
    sum_tk: f64,
    prev_sum_tk: f64,
    max_iterations: u32,
    ln_bailout: f64,
    output: f64,
    in_set: bool,
}
impl StripeAverageAlgorithm {
    const DEFAULT_BAILOUT2: f64 = 1e8;

    pub const DEFAULT_SKIP_ITERATION: u32 = 1;
    pub const DEFAULT_STRIPE_DENSITY: f64 = 1.0;

    fn factory(skip_iteration: u32, stripe_density: f64, bailout2: f64) -> Self {
        Self {
            bailout2,
            skip_iteration,
            stripe_density,
            sum_tk: 0.0,
            prev_sum_tk: Default::default(),
            max_iterations: Default::default(),
            ln_bailout: Default::default(),
            output: 0.0,
            in_set: false,
        }
    }

    pub fn new(skip_iteration: u32, stripe_density: f64) -> Self {
        Self::factory(skip_iteration, stripe_density, Self::DEFAULT_BAILOUT2)
    }

    /// t_n = t(z_n) = sin(s * arg(z_n))/2 + 1/2
    fn attend_double(&self, z: Complex) -> f64 {
        0.5 + f64::sin(self.stripe_density * z.arg()) * 0.5
    }

    fn attend_big(&self, z: &BigComplex) -> f64 {
        0.5 + f64::sin(self.stripe_density + z.arg()) * 0.5
    }

    fn final_interpolated_a(&self, abs2_z: f64, i: u32) -> f64 {
        let mut i = i;
        if i <= self.skip_iteration {
            i = self.skip_iteration + 1;
        }

        // A_{n, k}
        let avg_tk = self.sum_tk / (i - self.skip_iteration) as f64;
        // A_{n, k-1}
        let prev_avg_tk = self.prev_sum_tk / (i - self.skip_iteration - 1) as f64;

        // smooth iteration count
        let ln_zmod = f64::ln(abs2_z) * 0.5;
        let d = (i + 1) as f64 + f64::ln(self.ln_bailout / ln_zmod) / 0.69314718055994530942;
        let d = d % 1.0;

        let i_frac = d * avg_tk + (1.0 - d) * prev_avg_tk;
        i_frac * self.max_iterations as f64
    }
}
impl LayerAlgorithm for StripeAverageAlgorithm {
    fn get_bailout2(&self) -> f64 {
        self.bailout2
    }

    fn before(&mut self, max_iterations: u32) {
        self.max_iterations = max_iterations;
        self.ln_bailout = f64::ln(self.bailout2) * 0.5;
    }

    fn during_double(&mut self, z: Complex, i: u32) {
        if i > self.skip_iteration {
            self.prev_sum_tk = self.sum_tk;
            self.sum_tk += self.attend_double(z);
        }
    }

    fn during_big(&mut self, z: &BigComplex, i: u32) {
        if i > self.skip_iteration {
            self.prev_sum_tk = self.sum_tk;
            self.sum_tk += self.attend_big(z);
        }
    }

    fn out_set_double(&mut self, z: Complex, i: u32) {
        self.output = self.final_interpolated_a(z.abs_squared(), i);
    }

    fn out_set_big(&mut self, z: &BigComplex, i: u32) {
        self.output = self.final_interpolated_a(z.abs_squared(), i);
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

    fn get_in_set(&self) -> bool {
        self.in_set
    }
}
impl Default for StripeAverageAlgorithm {
    fn default() -> Self {
        Self::factory(
            Self::DEFAULT_SKIP_ITERATION,
            Self::DEFAULT_STRIPE_DENSITY,
            Self::DEFAULT_BAILOUT2,
        )
    }
}
