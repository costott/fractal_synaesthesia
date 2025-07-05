pub mod types;
use types::*;

pub mod shaders;
use shaders::*;

mod rendering;

use macroquad::prelude::*;

pub struct ReferenceOrbit {
    /// the reference orbit, starting from `0 + 0i`
    pub ref_z: Vec<Complex>,
    /// the iteration just before the referencre orbit diverged
    pub max_ref_iteration: usize
}
impl ReferenceOrbit {
    pub fn new(center: &BigComplex, max_iterations: usize, bailout2: f64) -> ReferenceOrbit {
        let mut ref_z: Vec<Complex> = Vec::with_capacity(max_iterations);
        let mut max_ref_iteration= 0;

        let mut z = BigComplex::from_f64s(0., 0.);
        for i in 0..max_iterations {
            ref_z.push(z.as_complex());
            if z.abs_squared() < bailout2 {
                z = &z.square() + center;
                max_ref_iteration = i;
            } else {
                break;
            }
        }
    
        ReferenceOrbit { ref_z, max_ref_iteration }
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

            image.get_image_data_mut()[4*i] = real_bytes[0..4].try_into().unwrap();
            image.get_image_data_mut()[4*i + 1] = real_bytes[4..8].try_into().unwrap();
            image.get_image_data_mut()[4*i + 2] = im_bytes[0..4].try_into().unwrap();
            image.get_image_data_mut()[4*i + 3] = im_bytes[4..8].try_into().unwrap();
        }

        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        texture
    }
}