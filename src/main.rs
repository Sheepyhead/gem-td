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

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use common::{BuildGrid, GameState};
use iyes_loopless::prelude::*;

mod common;
mod input;

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 300.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const CAMERA_OFFSET: [f32; 3] = [0.0, 12.0, 10.0];

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .insert_resource(ClearColor(CLEAR))
        // External plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: HEIGHT * RESOLUTION,
                height: HEIGHT,
                title: "Bevy Template".to_string(),
                resizable: false,
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin)
        // Internal plugins
        .add_loopless_state(GameState::InGame)
        .add_plugin(input::Input)
        .insert_resource(BuildGrid::default())
        .add_enter_system_set(
            GameState::InGame,
            ConditionSet::new()
                .with_system(spawn_camera)
                .with_system(spawn_ground)
                .into(),
        )
        .add_system_set(ConditionSet::new().run_in_state(GameState::InGame).into())
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera3dBundle::default();

    camera.transform.translation = CAMERA_OFFSET.into();
    camera.transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn(camera);
}

fn spawn_ground(mut commands: Commands, ass: Res<AssetServer>) {
    commands.spawn((
        SceneBundle {
            scene: ass.load("ground.glb#Scene0"),
            ..default()
        },
        Collider::cuboid(100.0, 0.01, 100.0),
    ));
}
