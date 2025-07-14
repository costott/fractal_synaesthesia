use dashu_float::{FBig, round::mode};
use macroquad::prelude::*;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Computes `n!`.
fn factorial(n: u32) -> u32 {
    let mut result = 1;
    for i in 2..=n {
        result *= i;
    }
    result
}

/// Computes `nCr`.
fn choose(n: u32, r: u32) -> u32 {
    assert!(n >= r);
    if r == 0 || r == n {
        return 1;
    }
    if r == 1 || r == n - 1 {
        return n;
    }
    factorial(n) / (factorial(r) * factorial(n - r))
}

/// Linear interpolation between f64s `a` and `b` with parameter `t`.
fn lerpf64(a: f64, b: f64, t: f64) -> f64 {
    let output = (1. - t) * a + t * b;
    // Check for accuracy loss
    match output == (1. - t) * a || output == t * b {
        false => output,
        // Not accurate enough, so use FBigs and reconvert back to f64s
        true => lerp_fbig(
            FBig::try_from(a).unwrap(),
            FBig::try_from(b).unwrap(),
            &FBig::try_from(t).unwrap(),
        )
        .to_f64()
        .value(),
    }
}

/// Linear interpolation of f64s `a^p` and `b^p` with parameter `t`.
fn lerpf64_pow(a: f64, b: f64, t: f64, p: f64) -> f64 {
    lerpf64(a, b, t.powf(p))
}

/// Linear interpolation between FBigs `a` and `b` with parameter `t`.
fn lerp_fbig(a: FBig, b: FBig, t: &FBig) -> FBig {
    (FBig::ONE - t) * a + t * b
}

/// Linear interpolation of FBigs `a^p` and `b^p` with parameter `t`.
fn lerp_fbig_pow(a: FBig, b: FBig, t: &FBig, p: &FBig) -> FBig {
    lerp_fbig(a, b, &t.powf(p))
}

/// Operations on complex numbers
pub trait ComplexNumber {
    /// Compute the square of this complex number (`self * self`)
    fn square(&self) -> Self;
    /// The absolute value of the complex number, squared
    fn abs_squared(&self) -> f64;
    /// The complex conjugate of the complex number
    fn conjugate(&self) -> Self;
    /// The 'argument' (angle between the positive real axis and the line joining the origin and this complex number)
    /// between `[-pi, pi]` inclusive.
    fn arg(&self) -> f64;
    /// The squared distance between the complex number and another
    fn distance2_to(&self, other: ComplexType) -> f64;
    /// Update the real part of this complex number from a given string.
    fn update_real_from_string(&mut self, new: String);
    /// Update the imaginary part of this complex number from a given string.
    fn update_im_from_string(&mut self, new: String);
    /// Returns the Complex number converted to a vec2.
    fn to_vec2(&self) -> Vec2;
    fn rotate(&self, angle: f64) -> Self;
}

/// An enum to hold a complex number of some (double/arbitrary precision) type
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexType {
    /// Double precision floating point numbers used for the real and imaginary parts.
    Double(Complex),
    /// Arbitrary precision floating point numbers used for the real and imaginary parts.
    Big(BigComplex),
}
impl ComplexType {
    /// Returns a complex number representing the `real` and `im` parts, of the same type as `other`.
    pub fn same_type(real: f64, im: f64, other: ComplexType) -> ComplexType {
        match other {
            ComplexType::Double(_) => ComplexType::Double(Complex::new(real, im)),
            ComplexType::Big(_) => ComplexType::Big(BigComplex::from_f64s(real, im)),
        }
    }

    /// Returns the real part of the number as an f64, regarless of type.
    pub fn real_f64(&self) -> f64 {
        match self {
            ComplexType::Double(c) => c.real,
            ComplexType::Big(c) => c.real.to_f64().value(),
        }
    }

    /// Returns the real part of the number as an FBig, regarless of type.
    pub fn real_fbig(&self) -> FBig {
        match self {
            ComplexType::Double(c) => FBig::try_from(c.real).unwrap(),
            ComplexType::Big(c) => c.real.clone(),
        }
    }

    /// Returns the real part of the number as a string, regardless of type.
    pub fn real_string(&self) -> String {
        match self {
            ComplexType::Double(c) => c.real.to_string(),
            ComplexType::Big(c) => c
                .real
                .clone()
                .with_base_and_precision::<10>(c.real.precision())
                .value()
                .to_string(),
        }
    }

    /// Returns the imaginary part of the number as an f64, regarless of type.
    pub fn im_f64(&self) -> f64 {
        match self {
            ComplexType::Double(c) => c.im,
            ComplexType::Big(c) => c.im.to_f64().value(),
        }
    }

    /// Returns the imaginary part of the number as an FBig, regarless of type.
    pub fn im_fbig(&self) -> FBig {
        match self {
            ComplexType::Double(c) => FBig::try_from(c.im).unwrap(),
            ComplexType::Big(c) => c.im.clone(),
        }
    }

    /// Returns the imaginary part of the number as a string, regardless of type.
    pub fn im_string(&self) -> String {
        match self {
            ComplexType::Double(c) => c.im.to_string(),
            ComplexType::Big(c) => {
                c.im.clone()
                    .with_base_and_precision::<10>(c.im.precision())
                    .value()
                    .to_string()
            }
        }
    }

    /// Converts the complex type to be [`ComplexType::Big`].
    pub fn make_big(&self) -> ComplexType {
        match &self {
            ComplexType::Double(c) => ComplexType::Big(BigComplex::from_complex(*c)),
            ComplexType::Big(_) => self.clone(),
        }
    }

    /// Converts the complex type to be [`ComplexType::Double`].
    pub fn make_double(&self) -> ComplexType {
        match &self {
            ComplexType::Big(c) => ComplexType::Double(c.as_complex()),
            ComplexType::Double(_) => self.clone(),
        }
    }

    /// Set the real part of this complex number to the given string `new`.
    pub fn update_real_from_string(&mut self, new: String) {
        match self {
            ComplexType::Double(c) => c.update_real_from_string(new.clone()),
            ComplexType::Big(c) => c.update_real_from_string(new.clone()),
        }
        // Preserve accuracy
        match self {
            // No accuracy preservation needed
            ComplexType::Big(_) => {}
            // If accuracy loss with Doubles, convert to a Big.
            ComplexType::Double(c) => {
                if c.real.to_string() != new {
                    *self = self.make_big();
                    self.update_real_from_string(new);
                }
            }
        }
    }
    pub fn update_im_from_string(&mut self, new: String) {
        match self {
            ComplexType::Double(c) => c.update_im_from_string(new.clone()),
            ComplexType::Big(c) => c.update_im_from_string(new.clone()),
        }
        // Preserve accuracy
        match self {
            // No accuracy preservation needed
            ComplexType::Big(_) => {}
            // If accuracy loss with Doubles, convert to a Big.
            ComplexType::Double(c) => {
                if c.im.to_string() != new {
                    *self = self.make_big();
                    self.update_im_from_string(new);
                }
            }
        }
    }

    /// Linear interpolation between two [`ComplexType`]s `complex1^p` and `complex2^p` with parameter `percent`.
    ///
    /// * `arb_precision` - whether or not to use arbitrary precision numbers for the interpolation.
    pub fn lerp_complex(
        complex1: &ComplexType,
        complex2: &ComplexType,
        percent: f64,
        arb_precision: bool,
        p: f64,
    ) -> ComplexType {
        match arb_precision {
            true => {
                let t = FBig::try_from(percent).unwrap();
                let p = FBig::try_from(p).unwrap();
                ComplexType::Big(BigComplex::new(
                    lerp_fbig_pow(complex1.real_fbig(), complex2.real_fbig(), &t, &p),
                    lerp_fbig_pow(complex1.im_fbig(), complex2.im_fbig(), &t, &p),
                ))
            }
            false => ComplexType::Double(Complex::new(
                lerpf64_pow(complex1.real_f64(), complex2.real_f64(), percent, p),
                lerpf64_pow(complex1.im_f64(), complex2.im_f64(), percent, p),
            )),
        }
    }
}

/// Complex number using f64s.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub im: f64,
}
impl Complex {
    pub fn new(real: f64, im: f64) -> Complex {
        Complex { real, im }
    }

    #[allow(unused)]
    /// raise the complex number to a given power
    /// TODO: use demoivre's instead
    pub fn pow(&self, n: u32) -> Self {
        let (mut real, mut im) = (0., 0.);
        for i in 0..=n {
            let b_pow = n - i;
            let coefficient =
                choose(n, i) as f64 * self.im.powi(b_pow as i32) * self.real.powi(i as i32);
            match b_pow % 4 {
                0 => real += coefficient,
                1 => im += coefficient,
                2 => real -= coefficient,
                3 => im -= coefficient,
                _ => {}
            }
        }

        Complex::new(real, im)
    }

    /// Returns the real part of the number as an f64.
    pub fn real_f64(&self) -> f64 {
        self.real
    }

    /// Returns the imaginary part of the number as an f64.
    pub fn im_f64(&self) -> f64 {
        self.im
    }
}
impl ComplexNumber for Complex {
    #[inline(always)]
    fn square(&self) -> Self {
        Complex::new(
            self.real * self.real - self.im * self.im,
            2f64 * self.real * self.im,
        )
    }

    #[inline(always)]
    fn abs_squared(&self) -> f64 {
        self.real * self.real + self.im * self.im
    }

    fn conjugate(&self) -> Complex {
        Complex {
            real: self.real,
            im: -self.im,
        }
    }

    fn arg(&self) -> f64 {
        f64::atan2(self.im, self.real)
    }

    fn distance2_to(&self, other: ComplexType) -> f64 {
        match other {
            ComplexType::Double(c) => (*self - c).abs_squared(),
            ComplexType::Big(c) => (BigComplex::from_complex(*self) - c).abs_squared(),
        }
    }

    fn update_real_from_string(&mut self, new: String) {
        if let Ok(new) = new.parse() {
            self.real = new;
        }
    }
    fn update_im_from_string(&mut self, new: String) {
        if let Ok(new) = new.parse() {
            self.im = new;
        }
    }

    fn to_vec2(&self) -> Vec2 {
        vec2(self.real as f32, self.im as f32)
    }

    #[inline(always)]
    fn rotate(&self, angle: f64) -> Self {
        Self {
            real: self.real * f64::cos(angle) - self.im * f64::sin(angle),
            im: self.real * f64::sin(angle) + self.im * f64::cos(angle),
        }
    }
}
impl Add for Complex {
    type Output = Complex;

    #[inline(always)]
    fn add(self, other: Complex) -> Complex {
        Complex {
            real: self.real + other.real,
            im: self.im + other.im,
        }
    }
}
impl Add<f64> for Complex {
    type Output = Complex;

    #[inline(always)]
    fn add(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real + rhs,
            im: self.im,
        }
    }
}
impl Sub for Complex {
    type Output = Complex;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real - rhs.real,
            im: self.im - rhs.im,
        }
    }
}
impl Sub<f64> for Complex {
    type Output = Complex;

    #[inline(always)]
    fn sub(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real - rhs,
            im: self.im,
        }
    }
}
impl Mul for Complex {
    type Output = Complex;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real * rhs.real - self.im * rhs.im,
            im: self.real * rhs.im + self.im * rhs.real,
        }
    }
}
impl Mul<f64> for Complex {
    type Output = Complex;

    #[inline(always)]
    fn mul(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real * rhs,
            im: self.im * rhs,
        }
    }
}
impl Div for Complex {
    type Output = Complex;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        let n = self * rhs.conjugate();
        let d = rhs.real * rhs.real + rhs.im * rhs.im;
        Complex {
            real: n.real / d,
            im: n.im / d,
        }
    }
}
impl Div<f64> for Complex {
    type Output = Complex;

    #[inline(always)]
    fn div(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real / rhs,
            im: self.im / rhs,
        }
    }
}
impl<'l> Div<f64> for &'l Complex {
    type Output = Complex;

    #[inline(always)]
    fn div(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real / rhs,
            im: self.im / rhs,
        }
    }
}

/// Complex number using FBigs.
#[derive(Debug, Clone, PartialEq)]
pub struct BigComplex {
    pub real: FBig,
    pub im: FBig,
}
impl BigComplex {
    pub fn new(real: FBig, im: FBig) -> BigComplex {
        BigComplex { real, im }
    }

    /// Creates a new BigComplex number from a double precision [`Complex`] number.
    pub fn from_complex(c: Complex) -> BigComplex {
        BigComplex {
            real: FBig::try_from(c.real).unwrap().with_precision(64).value(),
            im: FBig::try_from(c.im).unwrap().with_precision(64).value(),
        }
    }

    /// Creates a new BigComplex number from given f64 numbers.
    pub fn from_f64s(real: f64, im: f64) -> BigComplex {
        BigComplex {
            real: FBig::try_from(real).unwrap(),
            im: FBig::try_from(im).unwrap(),
        }
    }

    pub fn from_string_base10(real: &str, im: &str) -> BigComplex {
        // BigComplex {
        //     real: FBig::from_str_native(real).unwrap().with_precision(100).value(),
        //     im: FBig::from_str_native(im).unwrap().with_precision(100).value()
        // }
        BigComplex {
            real: FBig::<mode::Zero, 10>::from_str_native(real)
                .unwrap()
                .with_base::<2>()
                .value(),
            im: FBig::<mode::Zero, 10>::from_str_native(im)
                .unwrap()
                .with_base::<2>()
                .value(),
        }
    }

    /// Creates a new double precision [`Complex`] number from this BigComplex number
    pub fn as_complex(&self) -> Complex {
        Complex {
            real: self.real.to_f64().value(),
            im: self.im.to_f64().value(),
        }
    }

    /// Returns the real part of the number as an f64.
    pub fn real_f64(&self) -> f64 {
        self.real.to_f64().value()
    }

    /// Returns the imaginary part of the number as an f64.
    pub fn im_f64(&self) -> f64 {
        self.im.to_f64().value()
    }

    /// Ensures we're using the minimum precision for the desired number
    pub fn min_precision(&mut self) {
        fn get_min(current: FBig) -> usize {
            let mut min_precision = current.precision();

            while min_precision > 1 {
                let less_precision = current.clone().with_precision(min_precision - 1).value();

                if less_precision == current {
                    min_precision -= 1;
                } else {
                    min_precision += 1;
                    break;
                }
            }

            min_precision
        }

        self.real = self
            .real
            .clone()
            .with_precision(get_min(self.real.clone()))
            .value();
        self.im = self
            .im
            .clone()
            .with_precision(get_min(self.im.clone()))
            .value();
    }

    pub fn increase_precision(&self) -> Self {
        Self::new(
            self.real
                .clone()
                .with_precision(self.real.precision() * 2)
                .value(),
            self.im
                .clone()
                .with_precision(self.im.precision() * 2)
                .value(),
        )
    }

    /// Ensures accuracy is preserved during addition
    pub fn safe_add(&self, other: &Self) -> Self {
        let mut added = self.increase_precision() + other.increase_precision();
        added.min_precision();
        added
    }
}
impl ComplexNumber for BigComplex {
    fn square(&self) -> Self {
        BigComplex {
            real: self.real.sqr() - self.im.sqr(),
            im: (FBig::ONE + FBig::ONE) * &self.real * &self.im,
        }
    }

    fn abs_squared(&self) -> f64 {
        let abs = self.real.sqr() + self.im.sqr();
        abs.to_f64().value()
    }

    fn conjugate(&self) -> Self {
        BigComplex {
            real: self.real.clone(),
            im: self.im.clone().neg(),
        }
    }

    fn arg(&self) -> f64 {
        f64::atan2(self.im.to_f64().value(), self.real.to_f64().value())
    }

    fn distance2_to(&self, other: ComplexType) -> f64 {
        match other {
            ComplexType::Double(c) => (self.clone() - BigComplex::from_complex(c)).abs_squared(),
            ComplexType::Big(c) => (self.clone() - c).abs_squared(),
        }
    }

    fn update_real_from_string(&mut self, new: String) {
        if let Ok(new) = FBig::<mode::Zero, 10>::from_str_native(&new) {
            self.real = new.with_base::<2>().value();
        }
    }
    fn update_im_from_string(&mut self, new: String) {
        if let Ok(new) = FBig::<mode::Zero, 10>::from_str_native(&new) {
            self.im = new.with_base::<2>().value();
        }
    }

    fn to_vec2(&self) -> Vec2 {
        vec2(self.real_f64() as f32, self.im_f64() as f32)
    }

    fn rotate(&self, angle: f64) -> Self {
        let cosa = FBig::try_from(f64::cos(angle)).unwrap();
        let sina = FBig::try_from(f64::sin(angle)).unwrap();

        Self {
            real: self.real.clone() * cosa.clone() - self.im.clone() * sina.clone(),
            im: self.real.clone() * sina + self.im.clone() * cosa,
        }
    }
}
impl Add for BigComplex {
    type Output = BigComplex;

    fn add(self, other: BigComplex) -> Self::Output {
        BigComplex {
            real: self.real + other.real,
            im: self.im + other.im,
        }
    }
}
impl<'l, 'r> Add<&'r BigComplex> for &'l BigComplex {
    type Output = BigComplex;

    fn add(self, rhs: &'r BigComplex) -> Self::Output {
        BigComplex {
            real: &self.real + &rhs.real,
            im: &self.im + &rhs.im,
        }
    }
}
impl<'r> Add<&'r BigComplex> for BigComplex {
    type Output = BigComplex;

    fn add(self, rhs: &'r BigComplex) -> Self::Output {
        BigComplex {
            real: &self.real + &rhs.real,
            im: &self.im + &rhs.im,
        }
    }
}
impl Sub for BigComplex {
    type Output = BigComplex;

    fn sub(self, rhs: Self) -> Self::Output {
        BigComplex {
            real: self.real - rhs.real,
            im: self.im - rhs.im,
        }
    }
}
impl<'l, 'r> Sub<&'r BigComplex> for &'l BigComplex {
    type Output = BigComplex;

    fn sub(self, rhs: &'r BigComplex) -> Self::Output {
        BigComplex {
            real: &self.real - &rhs.real,
            im: &self.im - &rhs.im,
        }
    }
}
impl Mul for BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: Self) -> Self::Output {
        BigComplex {
            real: &self.real * &rhs.real - &self.im * &rhs.im,
            im: self.real * rhs.im + self.im * rhs.real,
        }
    }
}
impl<'l, 'r> Mul<&'r BigComplex> for &'l BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: &'r BigComplex) -> Self::Output {
        BigComplex {
            real: &self.real * &rhs.real - &self.im * &rhs.im,
            im: &self.real * &rhs.im + &self.im * &rhs.real,
        }
    }
}
impl<'l> Mul<BigComplex> for &'l BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: BigComplex) -> Self::Output {
        BigComplex {
            real: &self.real * &rhs.real - &self.im * &rhs.im,
            im: &self.real * &rhs.im + &self.im * &rhs.real,
        }
    }
}
impl Mul<f64> for BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: f64) -> Self::Output {
        let rhs = FBig::try_from(rhs).unwrap();
        BigComplex {
            real: self.real * &rhs,
            im: self.im * rhs,
        }
    }
}
impl<'l> Mul<f64> for &'l BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: f64) -> Self::Output {
        let rhs = FBig::try_from(rhs).unwrap();
        BigComplex {
            real: &self.real * &rhs,
            im: &self.im * rhs,
        }
    }
}
impl Mul<dashu_float::FBig> for BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: dashu_float::FBig) -> Self::Output {
        BigComplex {
            real: self.real * &rhs,
            im: self.im * rhs,
        }
    }
}
impl<'r> Mul<&'r dashu_float::FBig> for BigComplex {
    type Output = BigComplex;

    fn mul(self, rhs: &dashu_float::FBig) -> Self::Output {
        BigComplex {
            real: &self.real * rhs,
            im: &self.im * rhs,
        }
    }
}
impl Div for BigComplex {
    type Output = BigComplex;

    fn div(self, rhs: Self) -> Self::Output {
        let n = self * rhs.conjugate();
        let d = rhs.real.sqr() + rhs.im.sqr();
        BigComplex {
            real: n.real / &d,
            im: n.im / d,
        }
    }
}
impl<'l, 'r> Div<&'r BigComplex> for &'l BigComplex {
    type Output = BigComplex;

    fn div(self, rhs: &'r BigComplex) -> Self::Output {
        let n = self * &rhs.conjugate();
        let d = rhs.real.sqr() + rhs.im.sqr();
        BigComplex {
            real: n.real / &d,
            im: n.im / d,
        }
    }
}
impl Div<f64> for BigComplex {
    type Output = BigComplex;

    fn div(self, rhs: f64) -> Self::Output {
        let rhs = FBig::try_from(rhs).unwrap();
        BigComplex {
            real: self.real / &rhs,
            im: self.im / rhs,
        }
    }
}
impl<'l> Div<f64> for &'l BigComplex {
    type Output = BigComplex;

    fn div(self, rhs: f64) -> Self::Output {
        let rhs = FBig::try_from(rhs).unwrap();
        BigComplex {
            real: &self.real / &rhs,
            im: &self.im / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn create_big_complex() {
        let a = Complex::new(0.01, 1.23);
        let c = BigComplex::from_complex(a);

        assert_eq!(c.as_complex(), a);
    }

    #[test]
    fn add() {
        let a = Complex::new(1f64, 2f64);
        let b = Complex::new(3f64, 4f64);
        assert_eq!(a + b, Complex::new(4f64, 6f64));
    }

    #[test]
    fn add_bigcomplex() {
        let a = BigComplex::from_f64s(1f64, 2f64);
        let b = BigComplex::from_f64s(3f64, 4f64);
        assert_eq!(a + b, BigComplex::from_f64s(4f64, 6f64));
    }

    #[test]
    fn square() {
        let a = Complex::new(1f64, 2f64);
        assert_eq!(a.square(), Complex::new(-3f64, 4f64));
    }

    #[test]
    fn square_bigcomplex() {
        let a = BigComplex::from_f64s(1., 2.);
        let a2 = BigComplex::from_f64s(-3., 4.);
        assert_eq!(a.square(), a2);
    }

    #[test]
    fn arg() {
        let a = Complex::new(1.0, 1.0);
        let b = Complex::new(-1.0, 0.0);
        assert_eq!(a.arg(), PI / 4.);
        assert_eq!(b.arg(), PI);
    }

    #[test]
    fn power() {
        let a = Complex::new(2., -5.);
        let a3 = a.pow(3);

        assert_eq!(Complex::new(-142., 65.), a3);
    }

    #[test]
    fn complex_times() {
        let a = Complex::new(3., 5.);
        let b = Complex::new(2., 7.);

        let answer = Complex::new(-29., 31.);

        assert_eq!(a * b, answer);
    }

    #[test]
    fn bigcomplex_times() {
        let a = BigComplex::from_f64s(3., 5.);
        let b = BigComplex::from_f64s(2., 7.);

        let answer = BigComplex::from_f64s(-29., 31.);

        assert_eq!(a * b, answer);
    }

    #[test]
    fn complex_times_float() {
        let a = Complex::new(3., 6.);

        let answer = Complex::new(6., 12.);

        assert_eq!(a * 2., answer);
    }

    #[test]
    fn bigcomplex_times_float() {
        let a = BigComplex::from_f64s(3., 6.);

        let answer = BigComplex::from_f64s(6., 12.);

        assert_eq!(a * 2., answer);
    }

    #[test]
    fn complex_divide() {
        let a = Complex::new(3., 5.);
        let b = Complex::new(2., 4.);

        let answer = Complex::new(1.3, -0.1);

        assert_eq!(a / b, answer);
    }

    #[test]
    fn complex_divide_float() {
        let a = Complex::new(3., 18.);

        let answer = Complex::new(1., 6.);

        assert_eq!(a / 3., answer);
    }

    #[test]
    fn bigcomplex_divide_float() {
        let a = BigComplex::from_f64s(3., 18.);

        let answer = BigComplex::from_f64s(1., 6.);

        assert_eq!(a / 3., answer);
    }
}
