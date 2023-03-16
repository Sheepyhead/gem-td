use bevy::prelude::*;

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
