pub mod audio;
pub mod exporting;
pub mod rendering;
pub mod shaders;
pub mod types;
pub mod ui;

use macroquad::prelude::*;

use crate::ui::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "Fractal Synaesthesia".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();

    loop {
        app.update();
        app.draw();

        next_frame().await;
    }
}
