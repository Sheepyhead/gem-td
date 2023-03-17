use std::{fmt::Debug, time::Duration};

use bevy::{prelude::*, utils::HashSet};
use bevy_prototype_debug_lines::DebugLines;
use seldom_map_nav::prelude::*;

use crate::{
    creeps::{Damaged, HitPoints},
    CREEP_CLEARANCE, MAP_HEIGHT, MAP_WIDTH,
};

#[derive(Component)]
pub struct BasicTower;

#[derive(Component)]
pub struct Dirt;

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<Entity>);

impl BasicTower {
    pub fn update(
        mut lines: ResMut<DebugLines>,
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
                        lines.line_colored(
                            tower_pos.translation(),
                            target_pos.translation,
                            0.25,
                            Color::RED,
                        );

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

#[derive(Default, Deref, DerefMut, Resource)]
pub struct BuildGrid(HashSet<UVec2>);

impl Debug for BuildGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = String::with_capacity((MAP_WIDTH * MAP_HEIGHT + MAP_HEIGHT) as usize);
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                text.push(if self.contains(&UVec2::new(x, y)) {
                    'X'
                } else {
                    'O'
                });
            }
            text.push('\n');
        }
        write!(f, "{}", &text)
    }
}

pub fn uncover_dirt(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    dirt: Query<Entity, With<Dirt>>,
) {
    for entity in &dirt {
        let mut color: StandardMaterial =
            Color::rgba(fastrand::f32(), fastrand::f32(), fastrand::f32(), 0.5).into();
        color.alpha_mode = AlphaMode::Add;

        // Make a cooldown timer that starts in a finished state
        let time = 1.0;
        let mut timer = Timer::from_seconds(time, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(time));

        commands.entity(entity).insert((
            mats.add(color),
            BasicTower,
            Cooldown(timer),
            Target(None),
        ));
    }
}

pub fn rebuild_navmesh(
    mut commands: Commands,
    build_grid: Res<BuildGrid>,
    navmeshes: Query<Entity, With<Navmeshes>>,
) {
    let map = navmeshes.single();
    let mut tilemap = [Navability::Navable; ((MAP_WIDTH * MAP_HEIGHT) as usize)];
    for pos in dbg!(build_grid).iter() {
        tilemap[(pos.y * MAP_WIDTH + pos.x) as usize] = Navability::Solid;
    }
    let navability = |pos: UVec2| tilemap[(pos.y * MAP_WIDTH + pos.x) as usize];
    commands.entity(map).insert(
        Navmeshes::generate(
            [MAP_WIDTH, MAP_HEIGHT].into(),
            Vec2::new(1., 1.),
            navability,
            [CREEP_CLEARANCE],
        )
        .unwrap(),
    );
}
