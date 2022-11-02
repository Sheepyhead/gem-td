#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cargo_common_metadata,
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::multiple_crate_versions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::must_use_candidate,
    clippy::enum_glob_use
)]
#![feature(is_some_and)]

use bevy::{ecs::query::QuerySingleError, prelude::*, render::camera::Projection, utils::HashMap};
use bevy_rapier3d::prelude::*;
use common::approx_equal;
use debug::Debug;
use iyes_loopless::prelude::*;

mod common;
mod debug;

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const CAMERA_OFFSET: [f32; 3] = [0.0, 12.0, 10.0];

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy Template".to_string(),
            resizable: false,
            ..Default::default()
        })
        // External plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // Internal plugins
        .add_loopless_state(GameState::InGame)
        .add_plugin(Debug)
        .insert_resource(BuildGrid::default())
        .add_enter_system_set(
            GameState::InGame,
            ConditionSet::new()
                .with_system(spawn_camera)
                .with_system(spawn_ground)
                .with_system(spawn_dirt)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(update_selection_tile_color)
                .with_system(update_under_cursor)
                .with_system(move_selected)
                .into(),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera3dBundle::default();

    camera.transform.translation = CAMERA_OFFSET.into();
    camera.transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn_bundle(camera);
}

fn spawn_ground(mut commands: Commands, ass: Res<AssetServer>) {
    commands
        .spawn_bundle(SceneBundle {
            scene: ass.load("ground.glb#Scene0"),
            ..default()
        })
        .insert_bundle((Collider::cuboid(100.0, 0.01, 100.0),));
}

#[derive(Component)]
struct Selection;

fn update_under_cursor(
    mut commands: Commands,
    windows: Res<Windows>,
    context: Res<RapierContext>,
    camera: Query<(&bevy::prelude::Camera, &GlobalTransform, &Projection), With<Camera3d>>,
) {
    if let Some(cursor_pos_screen) = windows.get_primary().and_then(Window::cursor_position) {
        let (camera, camera_transform, projection) = camera.single();
        let projection = match projection {
            Projection::Perspective(persp) => persp,
            Projection::Orthographic(_) => panic!(),
        };
        let (from, to) = ray_from_screenspace(
            cursor_pos_screen,
            &windows,
            camera,
            projection,
            camera_transform,
            100.0,
        );

        if let Some((hit, RayIntersection { point, .. })) = context.cast_ray_and_get_normal(
            from,
            to,
            Real::MAX,
            false,
            QueryFilter::default().groups(InteractionGroups::all()),
        ) {
            commands.insert_resource(UnderCursor {
                _target: hit,
                intersection: point,
            });
        }
    }
}

struct UnderCursor {
    _target: Entity,
    intersection: Vec3,
}

pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    windows: &Res<Windows>,
    camera: &Camera,
    perspective: &PerspectiveProjection,
    camera_transform: &GlobalTransform,
    length: f32,
) -> (Vec3, Vec3) {
    let view = camera_transform.compute_matrix();
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
    let projection = camera.projection_matrix();

    // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
    let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;
    let is_orthographic = approx_equal(projection.w_axis[3], 1.0);

    // Compute the cursor position at the near plane. The bevy camera looks at -Z.
    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * perspective.near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    // Compute the ray's direction depending on the projection used.
    let ray_direction = if is_orthographic {
        view.transform_vector3(-Vec3::Z)
    } else {
        cursor_pos_near - camera_transform.translation()
    };

    (cursor_pos_near, ray_direction * length)
}

fn spawn_dirt(mut commands: Commands, ass: Res<AssetServer>) {
    commands
        .spawn_bundle(SceneBundle {
            scene: ass.load("dirtpile1.glb#Scene0"),
            ..default()
        })
        .insert_bundle((Name::new("Dirt Pile"),));
}

fn move_selected(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cursor: Option<Res<UnderCursor>>,
    mut selected: Query<&mut Transform, With<Selection>>,
) {
    match selected.get_single_mut() {
        Ok(mut selection) => {
            let cursor = match cursor {
                Some(cursor) => cursor,
                None => return,
            };
            selection.translation = [
                cursor.intersection.x.round(),
                selection.translation.y,
                cursor.intersection.z.round(),
            ]
            .into();
        }
        Err(QuerySingleError::NoEntities(..)) => {
            let cursor = match cursor {
                Some(cursor) => cursor,
                None => return,
            };
            commands
                .spawn_bundle(SpatialBundle {
                    transform: Transform::from_xyz(
                        cursor.intersection.x.round(),
                        0.1,
                        cursor.intersection.z.round(),
                    ),
                    ..default()
                })
                .insert(Selection)
                .with_children(|parent| {
                    let material = materials.add(Color::GREEN.into());
                    let mesh = meshes.add(shape::Plane { size: 1.0 }.into());
                    parent
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_translation((Vec3::X + Vec3::Z) / 2.0),
                            mesh: mesh.clone(),
                            material: material.clone(),
                            ..default()
                        })
                        .insert(SelectionTile);
                    parent
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_translation((Vec3::X + Vec3::NEG_Z) / 2.0),
                            mesh: mesh.clone(),
                            material: material.clone(),
                            ..default()
                        })
                        .insert(SelectionTile);
                    parent
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_translation((Vec3::NEG_X + Vec3::Z) / 2.0),
                            mesh: mesh.clone(),
                            material: material.clone(),
                            ..default()
                        })
                        .insert(SelectionTile);
                    parent
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_translation(
                                (Vec3::NEG_X + Vec3::NEG_Z) / 2.0,
                            ),
                            mesh: mesh.clone(),
                            material: material.clone(),
                            ..default()
                        })
                        .insert(SelectionTile);
                });
        }
        _ => panic!(),
    };
}

#[derive(Component)]
struct SelectionTile;

fn update_selection_tile_color(
    grid: Res<BuildGrid>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<
        (&GlobalTransform, &mut Handle<StandardMaterial>),
        (With<SelectionTile>, Changed<GlobalTransform>),
    >,
) {
    for (transform, mut material) in &mut tiles {
        *material = get_selection_tile_color(
            &grid,
            &transform,
            &materials.add(Color::GREEN.into()),
            &materials.add(Color::RED.into()),
        )
    }
}

fn get_selection_tile_color(
    grid: &BuildGrid,
    pos: &GlobalTransform,
    green: &Handle<StandardMaterial>,
    red: &Handle<StandardMaterial>,
) -> Handle<StandardMaterial> {
    if grid.contains(
        &(
            (pos.translation().x + 0.5).round() as i32,
            (pos.translation().z + 0.5).round() as i32,
        )
            .into(),
    ) {
        red
    } else {
        green
    }
    .clone()
}

#[derive(Default)]
struct BuildGrid(HashMap<IVec2, Entity>);

impl BuildGrid {
    fn insert(&mut self, pos: IVec2, content: Entity) {
        let positions = [pos, pos + IVec2::X, pos + IVec2::Y, pos + IVec2::ONE];
        if positions.iter().all(|pos| !self.contains(pos)) {
            for pos in positions {
                self.0.insert(pos, content);
            }
        }
    }

    fn contains(&self, pos: &IVec2) -> bool {
        self.0.contains_key(pos)
    }

    fn get(&self, pos: &IVec2) -> Option<Entity> {
        self.0.get(pos).copied()
    }

    fn remove(&mut self, pos: &IVec2) {
        if let Some(entity) = self.get(pos) {
            let possible_positions = [
                *pos + IVec2::NEG_X,
                *pos + IVec2::NEG_Y,
                *pos + IVec2::NEG_ONE,
                *pos + IVec2::X,
                *pos + IVec2::Y,
                *pos + IVec2::ONE,
                *pos + IVec2::NEG_X + IVec2::Y,
                *pos + IVec2::X + IVec2::NEG_Y,
            ];
            possible_positions
                .iter()
                .filter(|pos| {
                    self.get(*pos)
                        .is_some_and(|pos_entity| pos_entity == entity)
                })
                .copied()
                .collect::<Vec<_>>()
                .iter()
                .for_each(|pos| self.remove(pos));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    StartMenu,
    InGame,
}
