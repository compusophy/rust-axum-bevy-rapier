use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy_rapier2d::prelude::*;
use hexx::{Hex, HexLayout, Vec2 as HexVec2};

use crate::game::components::*;
use crate::game::resources::*;

// Custom Gizmo Group
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DashedGizmos;

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DashedGizmos>();
    config.line_style = GizmoLineStyle::Dotted;
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));
}

pub fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    if let Ok((mut transform, mut projection)) = query.get_single_mut() {
        let speed = 500.0 * time.delta_seconds();

        // Pan
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length_squared() > 0.0 {
            transform.translation += direction.normalize() * speed * projection.scale;
        }

        // Zoom
        let mut zoom_delta = 0.0;
        if keyboard_input.pressed(KeyCode::KeyQ) {
            zoom_delta += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            zoom_delta -= 1.0;
        }
        
        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    zoom_delta -= ev.y * 0.2;
                }
                MouseScrollUnit::Pixel => {
                    zoom_delta -= ev.y * 0.002;
                }
            }
        }

        if zoom_delta != 0.0 {
            projection.scale -= zoom_delta * time.delta_seconds();
            projection.scale = projection.scale.max(0.1);
        }
    }
}

pub fn spawn_units(
    mut commands: Commands,
    map_layout: Res<MapLayout>,
) {
    let layout = &map_layout.0;

    // Queen
    let queen_hex_pos = layout.hex_to_world_pos(Hex::ZERO);
    let queen_pos = Vec2::new(queen_hex_pos.x, queen_hex_pos.y);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb_u8(139, 69, 19), // SaddleBrown
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            transform: Transform::from_xyz(queen_pos.x, queen_pos.y, 1.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::ball(12.5),
        Ant,
        Queen,
    ));

    // Workers
    for hex in Hex::ZERO.ring(1).take(3) {
        let hex_pos = layout.hex_to_world_pos(hex);
        let pos = Vec2::new(hex_pos.x, hex_pos.y);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb_u8(139, 69, 19),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 1.0),
                ..default()
            },
            RigidBody::Dynamic,
            Sensor,
            Collider::ball(5.0),
            Damping { linear_damping: 20.0, angular_damping: 1.0 },
            Ant,
            Path::default(),
        ));
    }
}

pub fn handle_input(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut selection_state: ResMut<SelectionState>,
    ants_q: Query<(Entity, &Transform, Option<&Selected>), With<Ant>>,
    mut paths_q: Query<&mut Path>,
    map_layout: Res<MapLayout>,
) {
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };
    let Ok(window) = windows.get_single() else { return };

    let cursor_pos = if let Some(position) = window.cursor_position() {
        Some(position)
    } else {
        touches.first_pressed_position()
    };

    if let Some(screen_pos) = cursor_pos {
        let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else { return };
        
        // Drag Start
        if mouse_input.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
            selection_state.start_pos = Some(world_pos);
            selection_state.drag_current = Some(world_pos);
        }

        // Drag Update
        let any_touch = touches.iter().count() > 0;
        if mouse_input.pressed(MouseButton::Left) || any_touch {
            selection_state.drag_current = Some(world_pos);
        }

        // Drag End
        if mouse_input.just_released(MouseButton::Left) || touches.any_just_released() {
            let start = selection_state.start_pos.unwrap_or(world_pos);
            let dist = start.distance(world_pos);

            if dist < 5.0 {
                // Click
                handle_click(world_pos, &mut commands, &ants_q, &mut paths_q, &map_layout.0);
            } else {
                // Box Select
                handle_box_select(start, world_pos, &mut commands, &ants_q);
            }

            selection_state.start_pos = None;
            selection_state.drag_current = None;
        }
    }
}

fn handle_click(
    world_pos: Vec2,
    commands: &mut Commands,
    ants_q: &Query<(Entity, &Transform, Option<&Selected>), With<Ant>>,
    paths_q: &mut Query<&mut Path>,
    layout: &HexLayout,
) {
    // Check if clicked on a unit
    let mut clicked_unit = None;
    for (entity, transform, selected) in ants_q.iter() {
        if transform.translation.truncate().distance(world_pos) < 15.0 {
            clicked_unit = Some((entity, selected.is_some()));
            break;
        }
    }

    if let Some((entity, is_selected)) = clicked_unit {
        if is_selected {
            commands.entity(entity).remove::<Selected>();
        } else {
            commands.entity(entity).insert(Selected);
        }
    } else {
        // Move selected units
        let mut selected_units = Vec::new();
        for (entity, transform, selected) in ants_q.iter() {
            if selected.is_some() {
                selected_units.push((entity, transform.translation.truncate()));
            }
        }

        if !selected_units.is_empty() {
            calculate_paths(world_pos, selected_units, layout, paths_q);
        }
    }
}

fn handle_box_select(
    start: Vec2,
    end: Vec2,
    commands: &mut Commands,
    ants_q: &Query<(Entity, &Transform, Option<&Selected>), With<Ant>>,
) {
    let min = start.min(end);
    let max = start.max(end);
    let box_rect = Rect::from_corners(min, max);

    for (entity, transform, _) in ants_q.iter() {
        if box_rect.contains(transform.translation.truncate()) {
            commands.entity(entity).insert(Selected);
        }
    }
}

fn calculate_paths(
    target_world: Vec2,
    selected_units: Vec<(Entity, Vec2)>,
    layout: &HexLayout,
    paths_q: &mut Query<&mut Path>,
) {
    let target_hex = layout.world_pos_to_hex(HexVec2::new(target_world.x, target_world.y));
    let count = selected_units.len();
    let mut destinations = Vec::new();
    let mut allocated = 0;
    
    for hex in target_hex.spiral_range(0..10) {
        if allocated >= count { break; }
        destinations.push(hex);
        allocated += 1;
    }

    for (i, (entity, current_pos)) in selected_units.iter().enumerate() {
        if i >= destinations.len() { break; }
        let dest_hex = destinations[i];
        
        if let Ok(mut path) = paths_q.get_mut(*entity) {
            let current_hex = layout.world_pos_to_hex(HexVec2::new(current_pos.x, current_pos.y));
            let line = current_hex.line_to(dest_hex);
            
            path.waypoints.clear();
            for hex in line {
                 if hex == current_hex { continue; }
                 let pos = layout.hex_to_world_pos(hex);
                 path.waypoints.push_back(Vec2::new(pos.x, pos.y));
            }
            
            if path.waypoints.is_empty() && current_hex != dest_hex {
                 let pos = layout.hex_to_world_pos(dest_hex);
                 path.waypoints.push_back(Vec2::new(pos.x, pos.y));
            }
        }
    }
}

pub fn move_ants(
    mut commands: Commands,
    mut ants: Query<(Entity, &mut Transform, &mut Path, Option<&TargetPosition>), (With<Ant>, Without<Queen>)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let speed = 100.0;

    for (entity, mut transform, mut path, target_pos_opt) in ants.iter_mut() {
        let target = if let Some(tp) = target_pos_opt {
            Some(tp.0)
        } else if let Some(next) = path.waypoints.pop_front() {
            commands.entity(entity).insert(TargetPosition(next));
            Some(next)
        } else {
            None
        };

        if let Some(dest) = target {
            let current = transform.translation.truncate();
            let dist = current.distance(dest);
            
            if dist < 2.0 {
                transform.translation.x = dest.x;
                transform.translation.y = dest.y;
                commands.entity(entity).remove::<TargetPosition>();
            } else {
                let dir = (dest - current).normalize_or_zero();
                if dir != Vec2::ZERO {
                    let move_dist = speed * dt;
                    let new_pos = current + dir * move_dist;
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    
                    let angle = dir.y.atan2(dir.x);
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}

pub fn draw_selection_visuals(
    mut gizmos: Gizmos,
    mut dashed_gizmos: Gizmos<DashedGizmos>,
    ants: Query<(&Transform, &Path, Option<&Selected>), With<Ant>>,
    selection_state: Res<SelectionState>,
    map_layout: Res<MapLayout>,
) {
    let layout = &map_layout.0;

    if let (Some(start), Some(current)) = (selection_state.start_pos, selection_state.drag_current) {
        let min = start.min(current);
        let max = start.max(current);
        let center = (min + max) / 2.0;
        let size = max - min;
        gizmos.rect_2d(center, 0.0, size, Color::WHITE);
    }

    for (transform, path, selected) in ants.iter() {
        if selected.is_some() {
            let hex = layout.world_pos_to_hex(HexVec2::new(transform.translation.x, transform.translation.y));
            let corners = layout.hex_corners(hex);
            for i in 0..6 {
                let p1 = corners[i];
                let p2 = corners[(i + 1) % 6];
                gizmos.line_2d(Vec2::new(p1.x, p1.y), Vec2::new(p2.x, p2.y), Color::srgb(1.0, 1.0, 0.0));
            }

            let mut prev = transform.translation.truncate();
            for &wp in &path.waypoints {
                 dashed_gizmos.line_2d(prev, wp, Color::srgb(1.0, 1.0, 0.0));
                 prev = wp;
            }
            
            if let Some(target_pos) = path.waypoints.back() {
                 let dest_hex = layout.world_pos_to_hex(HexVec2::new(target_pos.x, target_pos.y));
                 let corners = layout.hex_corners(dest_hex);
                 for i in 0..6 {
                    let p1 = corners[i];
                    let p2 = corners[(i + 1) % 6];
                    dashed_gizmos.line_2d(Vec2::new(p1.x, p1.y), Vec2::new(p2.x, p2.y), Color::srgb(1.0, 1.0, 0.0));
                }
            }
        }
    }
}

pub fn draw_hex_grid(mut gizmos: Gizmos, map_layout: Res<MapLayout>) {
    let layout = &map_layout.0;
    for hex in Hex::ZERO.spiral_range(0..10) {
        let corners = layout.hex_corners(hex);
        for i in 0..6 {
            let p1 = corners[i];
            let p2 = corners[(i + 1) % 6];
            gizmos.line_2d(Vec2::new(p1.x, p1.y), Vec2::new(p2.x, p2.y), Color::srgb(0.5, 0.5, 0.5).with_alpha(0.2));
        }
    }
}
