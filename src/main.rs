mod ai;
mod game;
mod physics;
mod render;
mod terrain;

use macroquad::prelude::*;
use render::app::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "Lobber".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();

    loop {
        if app.should_quit {
            break;
        }

        if app.show_title {
            app.draw_title_screen();
            if is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Escape)
            {
                app.show_title = false;
            }
            next_frame().await;
            continue;
        }

        app.handle_input();
        app.update();
        app.render();

        next_frame().await;
    }
}
