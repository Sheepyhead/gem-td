use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_lyon::prelude::*;
use seldom_map_nav::prelude::{Navability, Navmeshes};

use crate::{
    common::Fadeout,
    controls::TileOccupied,
    creeps::{Damaged, HitPoints},
    CREEP_CLEARANCE, MAP_HEIGHT, MAP_WIDTH, TILE_SIZE,
};

#[derive(Component)]
pub struct BasicTower;

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<Entity>);

impl BasicTower {
    pub fn update(
        mut commands: Commands,
        time: Res<Time>,
        mut writer: EventWriter<Damaged>,
        mut towers: Query<(&mut Cooldown, &mut Target, &GlobalTransform), With<BasicTower>>,
        positions: Query<(Entity, &Transform), With<HitPoints>>,
    ) {
        for (mut cooldown, mut target, tower_pos) in &mut towers {
            cooldown.tick(time.delta());
            if cooldown.finished() {
                if let Some(target_entity) = **target {
                    // Tower has a target
                    if let Ok((_, target_pos)) = positions.get(target_entity) {
                        // Target is alive
                        cooldown.reset();
                        let beam =
                            shapes::Line(tower_pos.translation().xy(), target_pos.translation.xy());
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
                            target: target_entity,
                            value: 50,
                        });
                    } else {
                        // Target is dead
                        **target = None;
                    }
                } else {
                    // Tower needs to find a new target
                    let closest = Self::get_closest_creep(
                        positions
                            .iter()
                            .map(|(entity, position)| (entity, position.translation)),
                        tower_pos.translation(),
                    );

                    if let Some((creep, _)) = closest {
                        **target = Some(creep);
                    }
                }
            }
        }
    }

    fn get_closest_creep(
        creeps: impl Iterator<Item = (Entity, Vec3)>,
        position: Vec3,
    ) -> Option<(Entity, Vec3)> {
        let mut closest = None;
        let mut closest_distance_squared = f32::MAX;
        for (creep, creep_pos) in creeps {
            let distance_squared = creep_pos.distance_squared(position);
            if distance_squared < closest_distance_squared {
                closest = Some((creep, creep_pos));
                closest_distance_squared = distance_squared;
            }
        }
        closest
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Cooldown(pub Timer);

pub fn rebuild_navmesh(
    mut commands: Commands,
    navmeshes: Query<Entity, With<Navmeshes>>,
    occupied_tiles: Query<&TilePos, With<TileOccupied>>,
) {
    let map = navmeshes.single();
    let mut tilemap = [Navability::Navable; ((MAP_WIDTH * MAP_HEIGHT) as usize)];
    for pos in &occupied_tiles {
        // println!("Occupied {:?} ({:?})", pos, pos.center_in_world(size, typ));
        tilemap[(pos.y * MAP_WIDTH + pos.x) as usize] = Navability::Solid;
    }
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_WIDTH + pos.x) as usize];
    commands.entity(map).insert(
        Navmeshes::generate(
            [MAP_WIDTH, MAP_HEIGHT].into(),
            TILE_SIZE,
            navability,
            [CREEP_CLEARANCE],
        )
        .unwrap(),
    );
}
