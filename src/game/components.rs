use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Ant;

#[derive(Component)]
pub struct Queen;

#[derive(Component, Debug, Clone, Copy)]
pub struct TargetPosition(pub Vec2);

#[derive(Component, Debug, Default)]
pub struct Path {
    pub waypoints: VecDeque<Vec2>,
}

#[derive(Component)]
pub struct Selected;

