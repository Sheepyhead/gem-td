use bevy::{math::Vec4Swizzles, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use crate::common::{cursor_pos_in_world, CursorPos};

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

pub fn pick_tile_under_cursor(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    existing_highlights: Query<Entity, &TileHighlight>,
) {
    for entity in &existing_highlights {
        commands
            .entity(entity)
            .remove::<TileHighlight>()
            .insert(TileTextureIndex(2));
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in &tilemap_q {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec3 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            [
                (tile_pos.x, tile_pos.y),
                (tile_pos.x + 1, tile_pos.y),
                (tile_pos.x, tile_pos.y.saturating_sub(1)),
                (tile_pos.x + 1, tile_pos.y.saturating_sub(1)),
            ]
            .iter()
            .copied()
            .filter_map(|(x, y)| tile_storage.checked_get(&TilePos { x, y }))
            .for_each(|entity| {
                commands
                    .entity(entity)
                    .insert((TileHighlight::Valid, TileTextureIndex(1)));
            });
        }
    }
}
