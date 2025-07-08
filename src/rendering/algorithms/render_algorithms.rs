use std::sync::Arc;

use super::layer_algorithms::{LayerAlgorithm, LayerImplementation};
use crate::types::*;

use macroquad::prelude::*;

/// Fractal rendering algorithms
pub enum Fractal {
    Mandelbrot { power: u32 },
}
impl Fractal {
    pub fn iterate_big(&self, z: &mut BigComplex, c: &BigComplex) {
        *z = &z.square() + c;
    }

    pub fn iterate_perturbed(
        &self,
        reference_orbit: &ReferenceOrbit,
        ref_iteration: u32,
        dz: &mut Complex,
        dc: &Complex,
    ) {
        match *self {
            Self::Mandelbrot { power } => {
                if power == 2 {
                    *dz = reference_orbit.ref_z[ref_iteration as usize] * *dz * 2.0
                        + dz.square()
                        + *dc;
                } else {
                    todo!();
                }
            }
        }
    }
}

pub struct ReferenceOrbit {
    /// the reference orbit, starting from `0 + 0i`
    pub ref_z: Vec<Complex>,
    /// the iteration just before the referencre orbit diverged
    pub max_ref_iteration: u32,
}
impl ReferenceOrbit {
    pub fn new(
        fractal: &Fractal,
        center: &BigComplex,
        max_iterations: u32,
        bailout2: f64,
    ) -> ReferenceOrbit {
        let mut ref_z: Vec<Complex> = Vec::with_capacity(max_iterations as usize);
        let mut max_ref_iteration = 0;

        let mut z = BigComplex::from_f64s(0., 0.);
        for i in 0..max_iterations {
            ref_z.push(z.as_complex());
            if z.abs_squared() < bailout2 {
                fractal.iterate_big(&mut z, center);
                max_ref_iteration = i;
            } else {
                break;
            }
        }

        ReferenceOrbit {
            ref_z,
            max_ref_iteration,
        }
    }

    /// Encode the reference orbit into a texture to send to the GPU.
    ///
    /// # Format
    /// 2D texture with 1 row, where each group of 4 pixels encodes 1 orbit point as follows:
    /// * pixel 1 - first portion of real part
    /// * pixel 2 - last portion of real part
    /// * pixel 3 - first portion of imaginary part
    /// * pixel 4 - last portion of imaginary part
    ///
    /// Each part is a f64, encoded as a 2 colours of 4 bytes using little-endian byte order.
    pub fn as_texture(&self) -> Texture2D {
        let mut image = Image::gen_image_color(4 * self.ref_z.len() as u16, 1, BLANK);

        for (i, z) in self.ref_z.iter().enumerate() {
            // split real+im f64s into 8 bytes each for packing
            let real_bytes = z.real.to_le_bytes();
            let im_bytes = z.im.to_le_bytes();

            image.get_image_data_mut()[4 * i] = real_bytes[0..4].try_into().unwrap();
            image.get_image_data_mut()[4 * i + 1] = real_bytes[4..8].try_into().unwrap();
            image.get_image_data_mut()[4 * i + 2] = im_bytes[0..4].try_into().unwrap();
            image.get_image_data_mut()[4 * i + 3] = im_bytes[4..8].try_into().unwrap();
        }

        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        texture
    }
}

/// Analyse a pixel, letting the implementations calculate their outputs.
///
/// # Arguments
///
/// * `fractal` - The fractal algorithm to use.
/// * `dc` - A [`Complex`] number representing the vector between the
/// reference orbit point (centre of screen) and the pixel to analyse.
/// * `implementations` - The list of layering algorithms to run while analysing this pixel.
///
/// # Returns
///
/// Whether or not this point is in the fractal set or not.
pub fn analyse_pixel(
    fractal: &Arc<Fractal>,
    dc: Complex,
    reference_orbit: &Arc<ReferenceOrbit>,
    max_iterations: u32,
    bailout2: f64,
    implementations: &mut Vec<LayerImplementation>,
) -> bool {
    // Uses this reference orbit method:
    // https://fractalforums.org/index.php?topic=4360.msg29835#msg29835

    let mut dz = Complex::new(0.0, 0.0);
    let mut ref_iteration = 0;

    for im in implementations.iter_mut() {
        im.before(max_iterations, bailout2);
    }

    for i in 0..max_iterations {
        fractal.iterate_perturbed(reference_orbit, ref_iteration, &mut dz, &dc);
        ref_iteration += 1;

        let z = reference_orbit.ref_z[ref_iteration as usize] + dz;

        // Point escaped
        if z.abs_squared() > bailout2 {
            for im in implementations.iter_mut() {
                im.out_set_double(z, i);
            }
            return false;
        }

        // Rebase |z + dz| < |dz|
        if z.abs_squared() < dz.abs_squared() || ref_iteration >= reference_orbit.max_ref_iteration
        {
            dz = z;
            ref_iteration = 0;
        }
    }

    // Point stayed bounded
    for im in implementations.iter_mut() {
        im.in_set_double(reference_orbit.ref_z[ref_iteration as usize] + dz);
    }

    true
}
