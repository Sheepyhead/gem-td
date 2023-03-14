use std::time::Duration;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec4Swizzles,
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    common::{cursor_pos_in_world, Builds, CursorPos},
    towers::{BasicTower, Cooldown, Target},
    Phase,
};

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    windows: Query<&Window>,
    camera_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let window = windows.single();
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            *cursor_pos = CursorPos(cursor_pos_in_world(
                window,
                cursor_moved.position,
                cam_t,
                cam,
            ));
        }
    }
}

#[derive(Component)]
pub enum TileHighlight {
    Valid,
}
impl TileHighlight {
    pub fn reset_tile_highlights(
        mut commands: Commands,
        existing_highlights: Query<Entity, With<TileHighlight>>,
    ) {
        for entity in &existing_highlights {
            commands
                .entity(entity)
                .remove::<TileHighlight>()
                .insert(TileTextureIndex(2));
        }
    }
}
pub fn highlight_tile_under_cursor(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
    )>,
    existing_highlights: Query<Entity, With<TileHighlight>>,
) {
    for entity in &existing_highlights {
        commands
            .entity(entity)
            .remove::<TileHighlight>()
            .insert(TileTextureIndex(2));
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in &tilemap_q {
        {
            if let Some(tile_pos) =
                tile_from_cursor_pos(&cursor_pos, map_transform, *map_size, *grid_size, *map_type)
            {
                // Highlight the relevant tile's label

                if let Some(tiles) = get_square_from_tiles(tile_pos, tile_storage) {
                    for entity in &tiles {
                        commands
                            .entity(*entity)
                            .insert((TileHighlight::Valid, TileTextureIndex(1)));
                    }
                }
            }
        }
    }
}

pub fn build_on_click(
    mut commands: Commands,
    mut mouse: EventReader<MouseButtonInput>,
    mut builds: ResMut<Builds>,
    mut next_phase: ResMut<NextState<Phase>>,
    phase: Res<State<Phase>>,
    cursor_pos: Res<CursorPos>,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
    )>,
    transforms: Query<&TilePos, Without<TileStorage>>,
) {
    // If there are no more builds and the current phase is Build, change phase
    if **builds == 0 && phase.0 == Phase::Build {
        next_phase.set(Phase::Spawn);
        return;
    }

    for event in mouse.iter() {
        if let MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
        } = event
        {
            for (map_size, grid_size, map_type, tile_storage, map_transform) in &tilemap_q {
                if let Some(pos) = tile_from_cursor_pos(
                    &cursor_pos,
                    map_transform,
                    *map_size,
                    *grid_size,
                    *map_type,
                ) {
                    if let Some(tiles) = get_square_from_tiles(pos, tile_storage) {
                        if let Ok(tile_pos) = transforms.get(*tiles.first().unwrap()) {
                            let pos = tile_pos.center_in_world(grid_size, map_type).extend(0.0)
                                + Vec3::new(0.0, 16.0, 10.0)
                                + map_transform.translation();

                            // Make a cooldown timer that starts in a finished state
                            let mut timer = Timer::from_seconds(0.5, TimerMode::Once);
                            timer.tick(Duration::from_secs_f32(0.5));

                            commands.spawn((
                                SpriteBundle {
                                    texture: asset_server.load("chippedemerald.png"),
                                    transform: Transform::from_translation(pos),
                                    ..default()
                                },
                                BasicTower,
                                Cooldown(timer),
                                Target(None),
                            ));
                            **builds -= 1;
                            dbg!(**builds);
                        }
                    }
                }
            }
        }
    }
}

fn tile_from_cursor_pos(
    CursorPos(cursor_pos): &CursorPos,
    map_transform: &GlobalTransform,
    map_size: TilemapSize,
    grid_size: TilemapGridSize,
    map_type: TilemapType,
) -> Option<TilePos> {
    // We need to make sure that the cursor's world position is correct relative to the map
    // due to any map transformation.
    let cursor_in_map_pos: Vec2 = {
        // Extend the cursor_pos vec3 by 1.0
        let cursor_pos = Vec4::from((*cursor_pos, 1.0));
        let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
        cursor_in_map_pos.xy()
    };
    // Once we have a world position we can transform it into a possible tile position.
    TilePos::from_world_pos(&cursor_in_map_pos, &map_size, &grid_size, &map_type)
}

fn get_square_from_tiles(tile_pos: TilePos, tile_storage: &TileStorage) -> Option<Vec<Entity>> {
    let tiles = [
        (tile_pos.x, tile_pos.y),
        (tile_pos.x + 1, tile_pos.y),
        (tile_pos.x, tile_pos.y.saturating_sub(1)),
        (tile_pos.x + 1, tile_pos.y.saturating_sub(1)),
    ]
    .iter()
    .copied()
    .filter_map(|(x, y)| tile_storage.checked_get(&TilePos { x, y }))
    .collect::<Vec<_>>();
    if tiles.len() < 4 {
        None
    } else {
        Some(tiles)
    }
}
