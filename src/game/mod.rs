use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use resources::{MapLayout, SelectionState};
use systems::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MapLayout>()
            .init_resource::<SelectionState>()
            .init_gizmo_group::<DashedGizmos>()
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).in_schedule(Update))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, (setup_camera, spawn_units, setup_gizmos))
            .add_systems(Update, (
                camera_movement,
                handle_input,
                move_ants,
                draw_selection_visuals,
                draw_hex_grid,
            ));
    }
}

