use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::{
        shape::{Cube, Plane},
        *,
    },
    utils::HashSet,
};
use bevy_rapier3d::prelude::*;

use crate::{
    common::{ray_from_screenspace, Builds},
    towers::{BuildGrid, Dirt},
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
    under_cursor: Res<UnderCursor>,
    build_grid: Res<BuildGrid>,
    mut existing_highlights: Query<
        (Entity, &mut Handle<StandardMaterial>, &mut Transform),
        With<TileHighlight>,
    >,
) {
    if !under_cursor.is_changed() {
        return;
    }

    if let Some(position) = **under_cursor {
        let mut moved_existing = false;
        let positions = get_squares_from_pos(position);
        for (index, (_, mut mat, mut transform)) in
            (&mut existing_highlights).into_iter().enumerate()
        {
            moved_existing = true;
            let pos = positions[index];
            *transform = Transform::from_translation(pos.extend(transform.translation.y).xzy());
            *mat = mats.add(
                #[allow(clippy::cast_sign_loss)]
                if build_grid.contains(&UVec2::new(pos.x as u32, pos.y as u32)) {
                    Color::RED
                } else {
                    Color::YELLOW
                }
                .into(),
            );
        }

        if !moved_existing {
            (0..4).for_each(|index| {
                let pos = positions[index];
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(
                            Plane {
                                size: 1.0,
                                ..default()
                            }
                            .into(),
                        ),
                        material: mats.add(
                            #[allow(clippy::cast_sign_loss)]
                            if build_grid.contains(&UVec2::new(pos.x as u32, pos.y as u32)) {
                                Color::RED
                            } else {
                                Color::YELLOW
                            }
                            .into(),
                        ),
                        transform: Transform::from_xyz(pos.x, 0.001, pos.y),
                        ..default()
                    },
                    TileHighlight,
                ));
            });
        }
    } else {
        // No tiles under cursor so remove any existing highlights
        for (entity, _, _) in &existing_highlights {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn remove_highlight(mut commands: Commands, highlight: Query<Entity, With<TileHighlight>>) {
    for entity in &highlight {
        commands.entity(entity).despawn_recursive();
    }
}

fn get_squares_from_pos(position: Vec2) -> [Vec2; 4] {
    let top_corner_position = Vec2::new(position.x.ceil() - 0.5, position.y.ceil() - 0.5);
    [
        top_corner_position,
        Vec2::new(top_corner_position.x, top_corner_position.y + 1.),
        Vec2::new(top_corner_position.x + 1., top_corner_position.y),
        Vec2::new(top_corner_position.x + 1., top_corner_position.y + 1.),
    ]
}

pub fn build_on_click(
    mut commands: Commands,
    mut mouse: EventReader<MouseButtonInput>,
    mut builds: ResMut<Builds>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut build_grid: ResMut<BuildGrid>,
    phase: Res<State<Phase>>,
    cursor_pos: Res<UnderCursor>,
) {
    // If there are no more builds and the current phase is Build, change phase
    if **builds == 0 && phase.0 == Phase::Build {
        next_phase.set(Phase::Pick);
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
                #[allow(clippy::cast_sign_loss)]
                let positions = get_squares_from_pos(pos.xz())
                    .map(|pos| UVec2::new((pos.x - 0.5) as u32, (pos.y - 0.5) as u32));
                if build_grid
                    .intersection(&positions.iter().copied().collect::<HashSet<_>>())
                    .count()
                    > 0
                {
                    // Attempted to build on occupied square
                    continue;
                }
                for pos in &positions {
                    build_grid.insert(*pos);
                }

                let mut color: StandardMaterial =
                    Color::rgba(fastrand::f32(), fastrand::f32(), fastrand::f32(), 0.5).into();
                color.alpha_mode = AlphaMode::Add;
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cube { size: 2.0 }.into()),
                        material: mats.add(Color::ORANGE_RED.into()),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    Dirt,
                ));

                **builds -= 1;
            }
        }
    }
}

pub fn pick_building(
    mut commands: Commands,
    mut mouse: EventReader<MouseButtonInput>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut next_phase: ResMut<NextState<Phase>>,
    cursor_pos: Res<UnderCursor>,
    dirt: Query<(Entity, &GlobalTransform), With<Dirt>>,
) {
    for event in mouse.iter() {
        if let MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
        } = event
        {
            if let Some(cursor_pos) = **cursor_pos {
                let mut picked_tower = None;
                for (entity, transform) in &dirt {
                    if transform.translation().xz().distance(cursor_pos) <= 1.0 {
                        picked_tower = Some(entity);
                    }
                }

                if let Some(picked_tower) = picked_tower {
                    for (entity, transform) in &dirt {
                        if entity == picked_tower {
                            commands.entity(entity).remove::<Dirt>();
                        } else {
                            commands.entity(entity).despawn();
                            commands.spawn(PbrBundle {
                                mesh: meshes.add(Cube { size: 2.0 }.into()),
                                material: mats.add(Color::ORANGE_RED.into()),
                                transform: transform.compute_transform(),
                                ..default()
                            });
                        }
                    }
                    next_phase.set(Phase::Spawn);
                }
            }
        }
    }
}
