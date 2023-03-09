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

use bevy::{
    math::{Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    window::WindowResolution,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_lyon::prelude::*;
use common::{cursor_pos_in_world, CursorPos, MovingTo, TrackWorldObjectToScreenPosition};
use progress_bar::ProgressBar;

mod common;
mod progress_bar;

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
            fadeout,
            Damaged::consume,
            TrackWorldObjectToScreenPosition::track,
            MovingTo::move_to,
            update_health_bars,
            Dead::death,
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
    let mut timer = Timer::from_seconds(3.0, TimerMode::Repeating);
    timer.tick(Duration::from_secs_f32(2.99));

    let creep = commands
        .spawn((
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
        ))
        .id();

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("chippedemerald.png"),
            transform: Transform::from_xyz(0.0, 32.0, 10.0),
            ..default()
        },
        BasicTower,
        Cooldown(timer),
        Target(creep),
    ));

    let bar = ProgressBar::spawn(
        (WINDOW_HEIGHT * RESOLUTION / 2.0, WINDOW_HEIGHT / 2.0).into(),
        Color::GREEN,
        Color::RED,
        1.0,
        creep,
        &mut commands,
    );

    commands
        .entity(bar)
        .insert(TrackWorldObjectToScreenPosition {
            target: creep,
            offset: Vec2::new(0.0, 21.0),
        });
}

#[derive(Component)]
struct BasicTower;

#[derive(Component, Deref, DerefMut)]
struct Target(Entity);

impl BasicTower {
    pub fn update(
        mut commands: Commands,
        time: Res<Time>,
        mut writer: EventWriter<Damaged>,
        mut towers: Query<(&mut Cooldown, &Target, &Transform), With<BasicTower>>,
        positions: Query<&Transform, Without<BasicTower>>,
    ) {
        for (mut cooldown, target, tower_pos) in &mut towers {
            if cooldown.tick(time.delta()).just_finished() {
                if let Ok(target_pos) = positions.get(**target) {
                    let beam =
                        shapes::Line(tower_pos.translation.xy(), target_pos.translation.xy());
                    commands.spawn((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&beam),
                            transform: Transform::from_xyz(0.0, 0.0, 100.0),
                            ..default()
                        },
                        Stroke::new(Color::RED, 3.0),
                        Fadeout(Timer::from_seconds(0.25, TimerMode::Once)),
                    ));
                    writer.send(Damaged {
                        target: **target,
                        value: 25,
                    });
                }
            }
        }
    }
}

fn fadeout(mut commands: Commands, time: Res<Time>, mut fadeouts: Query<(Entity, &mut Fadeout)>) {
    for (entity, mut timer) in &mut fadeouts {
        if timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Cooldown(Timer);

#[derive(Component, Deref, DerefMut)]
struct Fadeout(Timer);

#[derive(Component)]
struct Creep;

struct Damaged {
    target: Entity,
    value: u32,
}

impl Damaged {
    fn consume(
        mut reader: EventReader<Damaged>,
        mut writer: EventWriter<Dead>,
        mut targets: Query<&mut HitPoints>,
    ) {
        for damaged in &mut reader {
            if let Ok(mut hitpoints) = targets.get_mut(damaged.target) {
                hitpoints.sub(damaged.value);
                if hitpoints.dead() {
                    writer.send(Dead(damaged.target));
                }
            }
        }
    }
}

#[derive(Component, Debug)]
struct HitPoints {
    max: u32,
    current: u32,
}

impl HitPoints {
    fn new(value: u32) -> Self {
        Self {
            max: value,
            current: value,
        }
    }

    fn sub(&mut self, value: u32) {
        self.current = self.current.checked_sub(value).unwrap_or(0);
    }

    fn dead(&self) -> bool {
        self.current == 0
    }

    fn ratio(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}
// We need to keep the cursor position updated based on any `CursorMoved` events.
fn update_cursor_pos(
    windows: Query<&Window>,
    camera_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let window = windows.single();
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            *cursor_pos = CursorPos(cursor_pos_in_world(
                window,
                cursor_moved.position,
                cam_t,
                cam,
            ));
        }
    }
}

#[derive(Component)]
enum TileHighlight {
    Valid,
}

fn pick_tile_under_cursor(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    existing_highlights: Query<Entity, &TileHighlight>,
) {
    for entity in &existing_highlights {
        commands
            .entity(entity)
            .remove::<TileHighlight>()
            .insert(TileTextureIndex(2));
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in &tilemap_q {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec3 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            [
                (tile_pos.x, tile_pos.y),
                (tile_pos.x + 1, tile_pos.y),
                (tile_pos.x, tile_pos.y.checked_sub(1).unwrap_or(0)),
                (tile_pos.x + 1, tile_pos.y.checked_sub(1).unwrap_or(0)),
            ]
            .iter()
            .copied()
            .flat_map(|(x, y)| tile_storage.checked_get(&TilePos { x, y }))
            .for_each(|entity| {
                commands
                    .entity(entity)
                    .insert((TileHighlight::Valid, TileTextureIndex(1)));
            });
        }
    }
}

fn update_health_bars(
    mut health_bars: Query<(&mut Style, &ProgressBar)>,
    hitpoints: Query<&HitPoints, Changed<HitPoints>>,
) {
    for (mut style, bar) in &mut health_bars {
        if let Ok(hitpoints) = hitpoints.get(bar.target) {
            style.size.width = Val::Percent(hitpoints.ratio() * 100.0);
        }
    }
}

#[derive(Deref, DerefMut)]
struct Dead(Entity);

impl Dead {
    fn death(
        mut commands: Commands,
        mut reader: EventReader<Dead>,
        bars: Query<(&ProgressBar, &Parent)>,
    ) {
        for Dead(dead) in reader.iter() {
            commands.entity(*dead).despawn_recursive();
            for (bar, parent) in bars.iter() {
                if bar.target == *dead {
                    commands.entity(**parent).despawn_recursive();
                }
            }
        }
    }
}
