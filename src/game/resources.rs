use bevy::prelude::*;
use hexx::{HexLayout, HexOrientation, Vec2 as HexVec2};

#[derive(Resource)]
pub struct MapLayout(pub HexLayout);

impl Default for MapLayout {
    fn default() -> Self {
        Self(HexLayout {
            scale: HexVec2::splat(20.0),
            orientation: HexOrientation::Pointy,
            ..default()
        })
    }
}

#[derive(Resource, Default)]
pub struct SelectionState {
    pub start_pos: Option<Vec2>,
    pub drag_current: Option<Vec2>,
}
