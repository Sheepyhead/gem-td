use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::{shape::Plane, *},
    utils::HashSet,
};
use bevy_rapier3d::prelude::*;

use crate::{
    common::{get_squares_from_pos, position_within_rect, ray_from_screenspace, Builds},
    gui::Sidebar,
    towers::{BuildGrid, FulfillableSpecialTowerRecipes, JustBuilt, Tower},
    Phase, WINDOW_HEIGHT,
};

pub fn update_under_cursor(
    context: Res<RapierContext>,
    mut under_cursor: ResMut<UnderCursor>,
    windows: Query<&Window, Changed<Window>>,
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

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CursorOverGui(bool);

pub fn cursor_over_gui(
    mut cursor_over_gui: ResMut<CursorOverGui>,
    gui: Query<(&Node, &GlobalTransform), With<Sidebar>>,
    windows: Query<&Window, Changed<Window>>,
) {
    for (node, transform) in &gui {
        if let Some(cursor_pos_screen) = windows.single().cursor_position() {
            **cursor_over_gui = position_within_rect(
                Vec2 {
                    x: cursor_pos_screen.x,
                    y: WINDOW_HEIGHT - cursor_pos_screen.y,
                },
                node.size(),
                transform.translation().xy(),
            );
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
    mut build_grid: ResMut<BuildGrid>,
    phase: Res<State<Phase>>,
    cursor_pos: Res<UnderCursor>,
    cursor_over_gui: Res<CursorOverGui>,
) {
    // If there are no more builds and the current phase is Build, change phase
    if **builds == 0 && phase.0 == Phase::Build {
        next_phase.set(Phase::Pick);
        return;
    }

    if **cursor_over_gui {
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
                        mesh: meshes.add(Tower::Dirt.into()),
                        material: mats.add(Color::ORANGE_RED.into()),
                        transform: Transform::from_xyz(pos.x, Tower::Dirt.get_y_offset(), pos.z),
                        ..default()
                    },
                    JustBuilt,
                    Tower::Dirt,
                ));

                **builds -= 1;
            }
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Resource)]
pub struct SelectedTower {
    pub tower: Entity,
    pub pickable: bool,
    pub refinable: bool,
    pub removable: bool,
    pub combinable: bool,
}

impl SelectedTower {
    pub fn selection(
        mut commands: Commands,
        mut mouse: EventReader<MouseButtonInput>,
        under_cursor: Res<UnderCursor>,
        cursor_over_gui: Res<CursorOverGui>,
        fulfillable_recipes: Res<FulfillableSpecialTowerRecipes>,
        towers: Query<(Entity, &GlobalTransform, &Tower)>,
        just_built: Query<(), With<JustBuilt>>,
    ) {
        for event in mouse.iter() {
            if let MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            } = event
            {
                if **cursor_over_gui {
                    continue;
                }
                if let Some(cursor_pos) = **under_cursor {
                    let mut picked_tower = None;
                    for (entity, transform, tower) in &towers {
                        if transform.translation().xz().distance(cursor_pos) <= 1.0 {
                            picked_tower = Some((entity, tower));
                        }
                    }

                    if let Some((picked_tower, typ)) = picked_tower {
                        let just_built = just_built.contains(picked_tower);
                        let pickable = just_built && *typ != Tower::Dirt;
                        commands.insert_resource(SelectedTower {
                            tower: picked_tower,
                            pickable,
                            refinable: towers
                                .iter()
                                .filter(|(_, _, tower)| *typ != Tower::Dirt && *tower == typ)
                                .count()
                                >= 2,
                            removable: !just_built
                                && towers
                                    .get_component::<Tower>(picked_tower)
                                    .is_ok_and(|tower| *tower == Tower::Dirt),
                            combinable: fulfillable_recipes
                                .iter()
                                .any(|recipe| recipe.ingredients.contains(typ)),
                        });
                    } else {
                        commands.remove_resource::<SelectedTower>();
                    }
                } else {
                    commands.remove_resource::<SelectedTower>();
                }
            }
        }
    }
}
