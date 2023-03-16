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

use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use common::{
    clamp_z_order_to_y, iso_pos_to_square_pos, tile_pos_to_square_pos, AvoidZOrderClamping, Builds,
    CursorPos, Fadeout, MovingTo, TrackWorldObjectToScreenPosition,
};
use controls::{build_on_click, highlight_tile_under_cursor, update_cursor_pos, TileHighlight};
use creeps::{CreepSpawner, Damaged, Dead, HitPoints};
use seldom_interop::prelude::Position2;
use seldom_map_nav::prelude::*;
use towers::{rebuild_navmesh, BasicTower};

mod common;
mod controls;
mod creeps;
mod progress_bar;
mod towers;

pub const CLEAR: Color = Color::BLACK;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const CAMERA_OFFSET: [f32; 3] = [0.0, 12.0, 10.0];
pub const CREEP_CLEARANCE: f32 = 16.0;

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
        .add_plugin(MapNavPlugin::<IsoPosition>::default())
        .add_plugin(WorldInspectorPlugin::new())
        // Internal plugins
        .add_state::<Phase>()
        .add_event::<Damaged>()
        .add_event::<Dead>()
        .init_resource::<CursorPos>()
        .init_resource::<Builds>()
        .add_startup_system(startup)
        .add_systems((
            update_cursor_pos,
            highlight_tile_under_cursor.in_set(OnUpdate(Phase::Build)),
            build_on_click.in_set(OnUpdate(Phase::Build)),
            BasicTower::update,
            Fadeout::fadeout,
            Damaged::consume,
            TrackWorldObjectToScreenPosition::track,
            MovingTo::move_to,
            Dead::death,
            CreepSpawner::spawn.in_set(OnUpdate(Phase::Spawn)),
            HitPoints::spawn_health_bars,
            HitPoints::update_health_bars,
            clamp_z_order_to_y,
            Builds::reset_system.in_schedule(OnEnter(Phase::Build)),
            TileHighlight::reset_tile_highlights.in_schedule(OnExit(Phase::Build)),
        ))
        .add_systems((
            CreepSpawner::reset_amount_system.in_schedule(OnEnter(Phase::Spawn)),
            check_state_change,
            rebuild_navmesh.in_schedule(OnExit(Phase::Build)),
            propagate_iso_navigation,
        ))
        .run();
}

const MAP_WIDTH: u32 = 4 * 4; // Originally 30 * 4
const MAP_HEIGHT: u32 = 4 * 4; // Originally 22 * 4
const TILE_SIZE: Vec2 = Vec2 { x: 64.0, y: 32.0 };

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = Camera2dBundle::new_with_far(20_000.0);
    camera.transform = Transform::from_xyz(
        MAP_WIDTH as f32 * TILE_SIZE.x / 2.0,
        MAP_HEIGHT as f32 / 2.0 * TILE_SIZE.y,
        10_000.0,
    );
    commands.spawn((camera, AvoidZOrderClamping));

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

    let tile_size = TilemapTileSize {
        x: TILE_SIZE.x,
        y: TILE_SIZE.y,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    let tilemap = [Navability::Navable; ((MAP_WIDTH * MAP_HEIGHT) as usize)];
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_WIDTH + pos.x) as usize];

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type,
            transform: Transform::from_xyz(
                TILE_SIZE.x / 2.0,
                MAP_HEIGHT as f32 / 2.0 * TILE_SIZE.y,
                -9_999.0,
            ),
            ..default()
        },
        AvoidZOrderClamping,
        Navmeshes::generate(
            [MAP_WIDTH, MAP_HEIGHT].into(),
            TILE_SIZE,
            navability,
            [CREEP_CLEARANCE],
        )
        .unwrap(),
    ));

    commands.spawn((
        CreepSpawner::default(),
        TransformBundle::from_transform(Transform::from_translation(
            tile_pos_to_square_pos(TilePos::new(0, 0)).extend(0.0),
        )),
    ));
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum Phase {
    #[default]
    Build,
    Spawn,
}

fn check_state_change(state: Res<State<Phase>>) {
    if state.is_changed() {
        // println!("State changed to {state:?}");
    }
}

#[derive(Component)]
struct IsoPosition {
    pos: Vec2,
}

impl Position2 for IsoPosition {
    type Position = Vec2;

    fn get(&self) -> Self::Position {
        self.pos
    }

    fn set(&mut self, pos: Self::Position) {
        self.pos = pos;
    }
}

fn propagate_iso_navigation(
    mut positions: Query<(&mut Transform, &IsoPosition), Changed<IsoPosition>>,
) {
    for (mut transform, IsoPosition { pos }) in &mut positions {
        transform.translation = iso_pos_to_square_pos(*pos).extend(0.0);
    }
}
