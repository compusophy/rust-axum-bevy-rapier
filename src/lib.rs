use bevy::prelude::*;
use bevy::window::WindowResolution;

pub mod game;

use game::GamePlugin;

pub fn run_app() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(1280.0, 720.0),
            title: "Ant Colony MMO".to_string(),
            canvas: Some("#bevy-canvas".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(GamePlugin);

    app.run();
}

