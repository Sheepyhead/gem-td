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

use bevy::prelude::*;
use debug::Debug;

mod debug;

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 10.0,
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
        // Internal plugins
        .add_plugin(Debug)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_ground)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera3dBundle::default();

    camera.transform.translation = Vec3::splat(10.0);
    camera.transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn_bundle(camera);
}

fn spawn_ground(mut commands: Commands, ass: Res<AssetServer>) {
    commands.spawn_bundle(SceneBundle {
        scene: ass.load("ground.glb#Scene0"),
        ..default()
    });
}
