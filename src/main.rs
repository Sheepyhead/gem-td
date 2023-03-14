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

use std::time::Duration;

use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_lyon::prelude::*;
use common::{CursorPos, Fadeout, MovingTo, TrackWorldObjectToScreenPosition};
use controls::{pick_tile_under_cursor, update_cursor_pos};
use creeps::{Creep, CreepSpawner, Damaged, Dead, HitPoints};
use towers::{BasicTower, Cooldown, Target};

mod common;
mod controls;
mod creeps;
mod progress_bar;
mod towers;

pub const CLEAR: Color = Color::BLACK;
pub const WINDOW_HEIGHT: f32 = 600.0;
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            WINDOW_HEIGHT * RESOLUTION,
                            WINDOW_HEIGHT,
                        ),

                        title: "GEM TD".to_string(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TilemapPlugin)
        .add_plugin(ShapePlugin)
        // Internal plugins
        .add_event::<Damaged>()
        .add_event::<Dead>()
        .init_resource::<CursorPos>()
        .add_startup_system(startup)
        .add_systems((
            update_cursor_pos,
            pick_tile_under_cursor,
            BasicTower::update,
            Fadeout::fadeout,
            Damaged::consume,
            TrackWorldObjectToScreenPosition::track,
            MovingTo::move_to,
            Dead::death,
            CreepSpawner::spawn,
            HitPoints::spawn_health_bars,
            HitPoints::update_health_bars,
        ))
        .run();
}

const MAP_WIDTH: u32 = 4 * 4; // Originally 30 * 4
const MAP_HEIGHT: u32 = 4 * 4; // Originally 22 * 4

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("iso_color.png");

    let map_size = TilemapSize {
        x: MAP_WIDTH,
        y: MAP_HEIGHT,
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTextureIndex(2),
        TilePos { x: 0, y: 0 },
        map_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 64.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });

    // Make a cooldown timer that starts in a finished state
    let mut timer = Timer::from_seconds(0.5, TimerMode::Once);
    timer.tick(Duration::from_secs_f32(0.5));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("creep.png"),
            transform: Transform::from_xyz(128.0, 0.0, 10.0),
            ..default()
        },
        Creep,
        HitPoints::new(100),
        MovingTo {
            destination: Vec2::splat(-100.0),
        },
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("chippedemerald.png"),
            transform: Transform::from_xyz(0.0, 32.0, 10.0),
            ..default()
        },
        BasicTower,
        Cooldown(timer),
        Target(None),
    ));

    commands.spawn((
        CreepSpawner(Timer::from_seconds(4.0, TimerMode::Repeating)),
        TransformBundle::from_transform(Transform::from_xyz(-100.0, -100.0, 100.0)),
    ));
}
