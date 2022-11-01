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

use bevy::{prelude::*, render::camera::Projection};
use bevy_rapier3d::prelude::*;
use common::approx_equal;
use debug::Debug;

mod common;
mod debug;

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const CAMERA_OFFSET: [f32; 3] = [10.0, 12.0, 10.0];

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
        .add_plugin(Debug)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_ground)
        .add_startup_system(spawn_dirt)
        .add_system(update_under_cursor)
        .add_system(move_selected)
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

fn spawn_dirt(mut commands: Commands, ass: Res<AssetServer>) {
    commands
        .spawn_bundle(SceneBundle {
            scene: ass.load("dirtpile1.glb#Scene0"),
            ..default()
        })
        .insert_bundle((Name::new("Dirt Pile"), Selected));
}

#[derive(Component)]
struct Selected;

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

fn move_selected(
    cursor: Option<Res<UnderCursor>>,
    mut selected: Query<&mut Transform, With<Selected>>,
) {
    let cursor = match cursor {
        Some(cursor) => cursor,
        None => return,
    };
    let mut selected = selected.single_mut();
    selected.translation = [
        cursor.intersection.x.round(),
        selected.translation.y,
        cursor.intersection.z.round(),
    ]
    .into();
}
