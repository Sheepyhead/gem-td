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

use bevy::{
    prelude::{shape::Plane, *},
    window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use common::{
    update_creep_position, Builds, CreepPos, Fadeout, MovingTo, TrackWorldObjectToScreenPosition,
};
use controls::{
    build_on_click, remove_highlight, show_highlight, update_under_cursor, SelectedTower,
    UnderCursor,
};
use creeps::{CreepSpawner, Dead, Hit, HitPoints, Slow};
use gui::GameGuiPlugin;
use seldom_map_nav::prelude::*;
use tower_abilities::TowerAbilitiesPlugin;
use towers::{
    rebuild_navmesh, uncover_dirt, BuildGrid, LaserAttack, PickTower, RandomLevel, RemoveTower,
    Upgrade, UpgradeAndPick,
};

mod common;
mod controls;
mod creeps;
mod gui;
mod progress_bar;
mod tower_abilities;
mod towers;

pub const CLEAR: Color = Color::BLACK;
pub const WINDOW_HEIGHT: f32 = 800.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const CAMERA_OFFSET: [f32; 3] = [0.0, 12.0, 10.0];
pub const CREEP_CLEARANCE: f32 = 0.25;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .insert_resource(ClearColor(CLEAR))
        // External plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            WINDOW_HEIGHT * RESOLUTION,
                            WINDOW_HEIGHT,
                        )
                        .with_scale_factor_override(1.),
                        title: "GEM TD".to_string(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(MapNavPlugin::<CreepPos>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // Internal plugins
        .add_state::<Phase>()
        .add_event::<Hit>()
        .add_event::<Dead>()
        .add_event::<PickTower>()
        .add_event::<RemoveTower>()
        .add_event::<UpgradeAndPick>()
        .add_event::<Upgrade>()
        .init_resource::<Builds>()
        .init_resource::<CurrentLevel>()
        .init_resource::<UnderCursor>()
        .init_resource::<BuildGrid>()
        .init_resource::<SelectedTower>()
        .init_resource::<RandomLevel>()
        .add_plugin(TowerAbilitiesPlugin)
        .add_plugin(GameGuiPlugin)
        .add_startup_system(startup)
        .add_systems((
            update_under_cursor,
            LaserAttack::attack,
            Fadeout::fadeout,
            Hit::consume,
            TrackWorldObjectToScreenPosition::track,
            MovingTo::move_to,
            Dead::death,
            CreepSpawner::spawn.in_set(OnUpdate(Phase::Spawn)),
            HitPoints::spawn_health_bars,
            HitPoints::update_health_bars,
            Builds::reset_system.in_schedule(OnEnter(Phase::Build)),
            show_highlight.in_set(OnUpdate(Phase::Build)),
            build_on_click.in_set(OnUpdate(Phase::Build)),
            update_creep_position,
        ))
        .add_systems((
            CreepSpawner::reset_amount_system.in_schedule(OnEnter(Phase::Spawn)),
            check_state_change,
            rebuild_navmesh.in_schedule(OnExit(Phase::Pick)),
            uncover_dirt.in_schedule(OnEnter(Phase::Pick)),
            remove_highlight.in_schedule(OnExit(Phase::Build)),
            PickTower::pick_building.in_set(OnUpdate(Phase::Pick)),
            next_level.in_schedule(OnExit(Phase::Spawn)),
            Slow::change.in_set(OnUpdate(Phase::Spawn)),
            LaserAttack::update_multiple_targets,
            SelectedTower::selection,
            RemoveTower::remove,
            UpgradeAndPick::upgrade_and_pick,
            Upgrade::upgrade,
        ))
        .run();
}

const MAP_WIDTH: u32 = 4 * 4; // Originally 30 * 4
const MAP_HEIGHT: u32 = 4 * 4; // Originally 22 * 4

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut build_grid: ResMut<BuildGrid>,
) {
    // Perfect isometric rotation
    let mut transform =
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0., 45_f32.to_radians(), 0.));
    transform.rotate_local_x(-35.264_f32.to_radians());
    // Imperfect camera placement wherever
    transform.translation = Vec3::new(
        MAP_WIDTH as f32 * 1.2,
        MAP_WIDTH as f32 * 0.5,
        MAP_HEIGHT as f32 * 1.2,
    );
    let camera = Camera3dBundle {
        transform,
        ..default()
    };
    commands.spawn(camera);

    let tilemap = [Navability::Navable; ((MAP_WIDTH * MAP_HEIGHT) as usize)];
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_WIDTH + pos.x) as usize];
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane {
                    size: MAP_WIDTH as f32,
                    ..default()
                }
                .into(),
            ),
            material: mats.add(Color::DARK_GREEN.into()),
            transform: Transform::from_xyz(MAP_WIDTH as f32 / 2., 0., MAP_HEIGHT as f32 / 2.),
            ..default()
        },
        Collider::cuboid(MAP_WIDTH as f32 / 2., 0.01, MAP_HEIGHT as f32 / 2.),
        Navmeshes::generate(
            [MAP_WIDTH, MAP_HEIGHT].into(),
            Vec2::new(1., 1.),
            navability,
            [CREEP_CLEARANCE],
        )
        .unwrap(),
    ));

    commands.spawn((CreepSpawner::default(),));

    build_grid.insert(UVec2::new(0, 15));
    build_grid.insert(UVec2::new(0, 14));
    build_grid.insert(UVec2::new(1, 14));
    build_grid.insert(UVec2::new(1, 15));
    build_grid.insert(UVec2::new(15, 0));
    build_grid.insert(UVec2::new(14, 0));
    build_grid.insert(UVec2::new(14, 1));
    build_grid.insert(UVec2::new(15, 1));
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum Phase {
    #[default]
    Build,
    Pick,
    Spawn,
}

fn check_state_change(state: Res<State<Phase>>) {
    if state.is_changed() {
        println!("State changed to {state:?}");
    }
}

#[derive(Resource, Clone, Copy, Deref, DerefMut)]
pub struct CurrentLevel(u32);

impl Default for CurrentLevel {
    fn default() -> Self {
        Self(1)
    }
}

pub fn next_level(mut level: ResMut<CurrentLevel>) {
    **level += 1;
}
