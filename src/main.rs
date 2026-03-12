mod app;
mod assets;
mod audio;
mod input;
mod model;
mod persistence;
mod physics;
mod render;
mod scene;

use app::App;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Gorilla".to_string(),
        window_width: 1920,
        window_height: 1080,
        high_dpi: true,
        fullscreen: false,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new().await;
    loop {
        app.update();
        app.draw();
        if app.should_quit() {
            break;
        }
        next_frame().await;
    }
}
