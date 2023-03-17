use std::{fmt::Debug, time::Duration};

use bevy::{prelude::*, utils::HashSet};
use bevy_prototype_debug_lines::DebugLines;
use seldom_map_nav::prelude::*;

use crate::{
    creeps::{Damaged, HitPoints},
    CREEP_CLEARANCE, MAP_HEIGHT, MAP_WIDTH,
};

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct Dirt;

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<Entity>);

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

        let typ = GemType::random();

        let gem_tower = GemTower { typ };
        commands.entity(entity).insert((
            mats.add(typ.into()),
            gem_tower,
            Tower,
            LaserAttack::from(gem_tower),
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
    for pos in build_grid.iter() {
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

#[derive(Clone, Copy)]
pub enum GemType {
    Emerald,
    Ruby,
    Sapphire,
    Diamond,
    Amethyst,
    Opal,
    Aquamarine,
    Topaz,
}

impl From<GemType> for StandardMaterial {
    fn from(val: GemType) -> Self {
        let mut color: StandardMaterial = Into::<Color>::into(val).into();
        color.alpha_mode = AlphaMode::Add;
        color
    }
}

impl From<GemType> for Color {
    fn from(value: GemType) -> Self {
        match value {
            GemType::Emerald => Color::GREEN,
            GemType::Ruby => Color::RED,
            GemType::Sapphire => Color::BLUE,
            GemType::Diamond => Color::WHITE,
            GemType::Amethyst => Color::PURPLE,
            GemType::Opal => Color::FUCHSIA,
            GemType::Aquamarine => Color::SEA_GREEN,
            GemType::Topaz => Color::YELLOW,
        }
    }
}

impl GemType {
    pub fn random() -> Self {
        match fastrand::u8(0..8) {
            0 => GemType::Emerald,
            1 => GemType::Ruby,
            2 => GemType::Sapphire,
            3 => GemType::Diamond,
            4 => GemType::Amethyst,
            5 => GemType::Opal,
            6 => GemType::Aquamarine,
            7 => GemType::Topaz,
            _ => panic!("Gem type larger than 6, this cannot happen"),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct GemTower {
    typ: GemType,
}

impl From<GemTower> for LaserAttack {
    fn from(value: GemTower) -> Self {
        Self {
            range: match value.typ {
                GemType::Ruby
                | GemType::Emerald
                | GemType::Diamond
                | GemType::Amethyst
                | GemType::Opal => 5.,
                GemType::Sapphire | GemType::Topaz => 6.,
                GemType::Aquamarine => 4.,
            },
            color: value.typ.into(),
            damage: match value.typ {
                GemType::Ruby => 4,
                GemType::Sapphire | GemType::Emerald | GemType::Amethyst | GemType::Aquamarine => 2,
                GemType::Diamond => 5,
                GemType::Opal => 1,
                GemType::Topaz => 3,
            },
        }
    }
}

#[derive(Component)]
pub struct LaserAttack {
    range: f32,
    color: Color,
    damage: u32,
}

impl LaserAttack {
    pub fn attack(
        mut lines: ResMut<DebugLines>,
        time: Res<Time>,
        mut writer: EventWriter<Damaged>,
        mut towers: Query<
            (&mut Cooldown, &mut Target, &GlobalTransform, &LaserAttack),
            With<Tower>,
        >,
        positions: Query<(Entity, &Transform), With<HitPoints>>,
    ) {
        for (mut cooldown, mut target, tower_pos, attack) in &mut towers {
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
                            attack.color,
                        );

                        writer.send(Damaged {
                            target: target_entity,
                            value: attack.damage,
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

                    if let Some((creep, distance)) = closest {
                        if dbg!(attack.range.powf(2.)) >= dbg!(distance) {
                            **target = Some(creep);
                        }
                    }
                }
            }
        }
    }

    fn get_closest_creep(
        creeps: impl Iterator<Item = (Entity, Vec3)>,
        position: Vec3,
    ) -> Option<(Entity, f32)> {
        let mut closest = None;
        let mut closest_distance_squared = f32::MAX;
        for (creep, creep_pos) in creeps {
            let distance_squared = creep_pos.distance_squared(position);
            if distance_squared < closest_distance_squared {
                closest = Some((creep, distance_squared));
                closest_distance_squared = distance_squared;
            }
        }
        closest
    }
}
