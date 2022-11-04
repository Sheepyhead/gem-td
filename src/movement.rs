use bevy::{math::Vec3Swizzles, prelude::*};

pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement);
    }
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct PathToFollow(pub Vec<Vec2>);

fn movement(time: Res<Time>, mut movers: Query<(&mut Transform, &Speed, &mut PathToFollow)>) {
    let delta_seconds = time.delta_seconds();
    for (mut t, v, mut path) in &mut movers {
        let Some(to) = path.0.first() else {
            continue;
        };
        let from = t.translation.xz();
        t.translation = move_towards(from, *to, v.0 * delta_seconds)
            .extend(0f32)
            .xzy();
        if t.translation.xz() == *to {
            path.0.remove(0);
        }
    }
}
pub fn move_towards(current: Vec2, target: Vec2, max_distance_delta: f32) -> Vec2 {
    let to_vector = target - current;

    let sqdist = target.distance_squared(current);

    if sqdist == 0.0 || (max_distance_delta >= 0.0 && sqdist <= max_distance_delta.powf(2.0)) {
        return target;
    }
    let dist = sqdist.sqrt();
    current + to_vector / dist * max_distance_delta
}
