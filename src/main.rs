use macroquad::prelude::*;
use fractal_synaesthesia::shaders::*;

use dashu_float::FBig;
use fractal_synaesthesia::types::BigComplex;
use fractal_synaesthesia::ReferenceOrbit;

fn window_conf() -> Conf {
    Conf {
        window_title: "Fractal Synaesthesia".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut center = BigComplex::from_f64s(
        -0.5,
        0.
    );
    let max_iterations = 500;
    let bailout2 = 100.;
    let mut pixel_step = 0.005;
    let mut velocity = 1.0;

    let render_target = render_target(screen_width() as u32, screen_height() as u32);
    let texture = render_target.texture.clone();
    texture.set_filter(FilterMode::Nearest);

    let camera = Camera2D {
            target: vec2(0., 0.),
            zoom: vec2(1., -1.),
            render_target: Some(render_target),
            ..Default::default()
    };

    let material = get_pipeline_material().await;

    loop {
        let dt = get_frame_time() as f64;

        if is_key_down(KeyCode::T) {
            pixel_step = 1.0;
            velocity = 1.0;
        }
        if is_key_down(KeyCode::Up) {
            pixel_step *= 1.0 - 0.5 / (get_fps() as f64);
            velocity *= 1.0 - 0.5 / (get_fps() as f64);
        } 
        if is_key_down(KeyCode::Down) {
            pixel_step *= 1.0 + 0.5 / (get_fps() as f64);
            velocity *= 1.0 + 0.5 / (get_fps() as f64);
        }
        
        let (mut moved_y, mut moved_x) = (true, true);
        let mut mod_center = center.clone();
        let movement = FBig::try_from(velocity * dt).unwrap().with_precision(0).value();
        let old_real = mod_center.real.clone();
        mod_center.real += movement.clone() * FBig::try_from(match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => {moved_x = false; 0.0}
        }).unwrap();
        if moved_x && old_real == mod_center.real {
            mod_center.real = mod_center.real.with_precision(old_real.precision()+1).value();
        }

        let old_im = mod_center.im.clone();
        mod_center.im += movement * FBig::try_from(match (is_key_down(KeyCode::W), is_key_down(KeyCode::S)) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => {moved_y = false; 0.0}
        }).unwrap();
        if moved_y && old_im == mod_center.im {
            mod_center.im = mod_center.im.with_precision(old_im.precision()+1).value();
        }

        center = mod_center;


        set_camera(&camera);

        clear_background(BLACK);

        let reference_orbit = ReferenceOrbit::new(&center, max_iterations, bailout2);

        gl_use_material(&material);
        set_pipleine_material_uniforms(
            &material, 
            &center, 
            reference_orbit.max_ref_iteration, 
            max_iterations, 
            bailout2, 
            pixel_step, 
            (screen_width(), screen_height())
        );
        material.set_texture("orbit_texture", reference_orbit.as_texture());

        // draw something so the camera knows to render
        draw_rectangle(-1.0, -1.0, 2.0, 2.0, WHITE);
        gl_use_default_material();

        // Switch back to default camera to draw the texture
        set_default_camera();
        draw_texture_ex(
            &texture,
            0., 0., 
            WHITE,
            DrawTextureParams { 
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        // draw_texture_ex(
        //     &reference_orbit.as_texture(), 
        //     0., 0., 
        //     WHITE, 
        //     DrawTextureParams { 
        //         dest_size: Some(vec2(screen_width(), screen_height())),
        //         ..Default::default()
        //     },
        // );
        draw_text(
            &format!("center: {:?}\nzoom: {}", center.as_complex(), 0.005 / pixel_step),
            0., 15., 25., BLUE
        );

        next_frame().await;
    }
}