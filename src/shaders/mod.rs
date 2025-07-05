use std::path::Path;

use macroquad::prelude::*;
use crate::types::{BigComplex, ComplexNumber};

/// Load a shader at a given `path`.
pub async fn load_shader_file(path: &Path) -> Result<String, macroquad::Error> {
    let bytes = load_file(path.to_str().unwrap()).await?;
    Ok(String::from_utf8(bytes).unwrap_or_else(|_| panic!("Shader file {:?} is not valid UTF-8", path)))
}

/// Returns the material used for the render pipeline.
pub async fn get_pipeline_material() -> macroquad::material::Material {
    // Load shaders from files
    let vertex_shader = load_shader_file(Path::new("./src/shaders/mandelbrot.vert"))
        .await
        .expect("Failed to load vertex shader");
    let fragment_shader = load_shader_file(Path::new("./src/shaders/perturbation.frag"))
        .await
        .expect("Failed to load fragment shader");

    // Load the shader material
    let material = load_material(
        ShaderSource::Glsl {
            vertex: &vertex_shader,
            fragment: &fragment_shader,
        },
        MaterialParams {
            textures: vec!["orbit_texture".to_string()],
            uniforms: vec![
                UniformDesc {
                    name: "center".to_owned(),
                    uniform_type: UniformType::Float2,
                    array_count: 1,
                },
                UniformDesc {
                    name: "max_ref_iterations".to_owned(),
                    uniform_type: UniformType::Int1,
                    array_count: 1,
                },
                UniformDesc {
                    name: "max_iterations".to_owned(),
                    uniform_type: UniformType::Int1,
                    array_count: 1,
                },
                UniformDesc {
                    name: "bailout2".to_owned(),
                    uniform_type: UniformType::Float1,
                    array_count: 1,
                },
                UniformDesc {
                    name: "pixel_step".to_owned(),
                    uniform_type: UniformType::Float1,
                    array_count: 1,
                },
                UniformDesc {
                    name: "dimensions".to_owned(),
                    uniform_type: UniformType::Float2,
                    array_count: 1,
                },
            ],
            ..Default::default()
        },
    )
    .unwrap();
    material
}

pub fn set_pipleine_material_uniforms(
    material: &Material,
    center: &BigComplex,
    max_ref_iterations: usize,
    max_iterations: usize,
    bailout2: f64,
    pixel_step: f64,
    dimensions: (f32, f32),
) {
    material.set_uniform("center", center.to_vec2());
    material.set_uniform("max_ref_iterations", max_ref_iterations as u32);
    material.set_uniform("max_iterations", max_iterations as u32);
    material.set_uniform("bailout2", bailout2 as f32);
    material.set_uniform("pixel_step", pixel_step as f32);
    material.set_uniform("dimensions", vec2(dimensions.0, dimensions.1));
}