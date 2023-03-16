use std::time::Duration;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::{
        shape::{Cube, Plane},
        *,
    },
};
use bevy_rapier3d::prelude::*;

use crate::{
    common::{ray_from_screenspace, Builds},
    towers::{BasicTower, Cooldown, Target},
    Phase,
};

pub fn update_under_cursor(
    context: Res<RapierContext>,
    mut under_cursor: ResMut<UnderCursor>,
    windows: Query<&Window>,
    camera: Query<(&bevy::prelude::Camera, &Projection, &GlobalTransform), With<Camera3d>>,
) {
    if let Some(cursor_pos_screen) = windows.single().cursor_position() {
        let (camera, projection, camera_transform) = camera.single();
        if let Projection::Perspective(projection) = projection {
            let (from, to) = ray_from_screenspace(
                cursor_pos_screen,
                windows.single(),
                camera,
                projection,
                camera_transform,
                100.0,
            );

            if let Some((_, RayIntersection { point, .. })) = context.cast_ray_and_get_normal(
                from,
                to,
                Real::MAX,
                false,
                QueryFilter::default().groups(CollisionGroups::default()),
            ) {
                **under_cursor = Some(point.xz());
            } else {
                **under_cursor = None;
            }
        }
    }
}

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct UnderCursor(pub Option<Vec2>);

#[derive(Component)]
pub struct TileHighlight;

pub fn show_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut existing_highlights: Query<(Entity, &mut Transform), With<TileHighlight>>,
    under_cursor: Res<UnderCursor>,
) {
    if !under_cursor.is_changed() {
        return;
    }

    if let Some(position) = **under_cursor {
        let mut moved_existing = false;
        let top_corner_position = Vec2::new(position.x.ceil() - 0.5, position.y.ceil() - 0.5);
        let positions = [
            top_corner_position,
            Vec2::new(top_corner_position.x, top_corner_position.y + 1.),
            Vec2::new(top_corner_position.x + 1., top_corner_position.y),
            Vec2::new(top_corner_position.x + 1., top_corner_position.y + 1.),
        ];
        for (index, (_, mut transform)) in (&mut existing_highlights).into_iter().enumerate() {
            moved_existing = true;
            *transform =
                Transform::from_translation(positions[index].extend(transform.translation.y).xzy());
        }

        if !moved_existing {
            (0..4).for_each(|index| {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(
                            Plane {
                                size: 1.0,
                                ..default()
                            }
                            .into(),
                        ),
                        material: mats.add(Color::PINK.into()),
                        transform: Transform::from_xyz(
                            positions[index].x,
                            0.001,
                            positions[index].y,
                        ),
                        ..default()
                    },
                    TileHighlight,
                ));
            });
        }
    } else {
        // No tiles under cursor so remove any existing highlights
        for (entity, _) in &existing_highlights {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn build_on_click(
    mut commands: Commands,
    mut mouse: EventReader<MouseButtonInput>,
    mut builds: ResMut<Builds>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    phase: Res<State<Phase>>,
    cursor_pos: Res<UnderCursor>,
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
            if let Some(cursor_pos) = **cursor_pos {
                let pos = Vec2::new(cursor_pos.x.ceil(), cursor_pos.y.ceil())
                    .extend(1.0)
                    .xzy();

                // Make a cooldown timer that starts in a finished state
                let time = 1.0;
                let mut timer = Timer::from_seconds(time, TimerMode::Once);
                timer.tick(Duration::from_secs_f32(time));

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cube { size: 2.0 }.into()),
                        material: mats.add(
                            Color::rgb(fastrand::f32(), fastrand::f32(), fastrand::f32()).into(),
                        ),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    BasicTower,
                    Cooldown(timer),
                    Target(None),
                ));
                **builds -= 1;
            }
        }
    }
}
