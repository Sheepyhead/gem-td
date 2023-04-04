use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
    time::Duration,
};

use bevy::{ecs::system::EntityCommands, math::Vec3Swizzles, prelude::*, utils::HashSet};
use bevy_prototype_debug_lines::DebugLines;
use seldom_map_nav::prelude::*;

use crate::{
    common::get_squares_from_pos,
    controls::SelectedTower,
    creeps::{Creep, CreepType, Hit, HitPoints},
    tower_abilities::{
        Aura, AuraType, CritOnHit, SapphireSlowOnHit, SlowPoisonOnHit, SpeedModifiers, SplashOnHit,
    },
    Phase, CREEP_CLEARANCE, MAP_HEIGHT, MAP_WIDTH,
};

pub const BASE_TOWER_SPEED: f32 = 1.0;

#[derive(Component, Clone)]
pub enum Target {
    Single(Option<Entity>),
    Multiple(Vec<Entity>),
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
    mut meshes: ResMut<Assets<Mesh>>,
    random_level: Res<RandomLevel>,
    just_built: Query<(Entity, &GlobalTransform), With<JustBuilt>>,
) {
    for (entity, pos) in &just_built {
        let typ = GemType::random();
        let gem_tower = Tower::GemTower {
            typ,
            quality: GemQuality::random_with_modifier(**random_level),
        };
        let cooldown: Cooldown = gem_tower.into();
        gem_tower.add_abilities(commands.entity(entity).insert((
            meshes.add(gem_tower.into()),
            Transform::from_xyz(
                pos.compute_transform().translation.x,
                gem_tower.get_y_offset(),
                pos.compute_transform().translation.z,
            ),
            mats.add(typ.into()),
            gem_tower,
            Name::new(gem_tower.to_string()),
            LaserAttack::from(gem_tower),
            cooldown,
            Target::Single(None),
            SpeedModifiers::default(),
        )));
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GemQuality {
    Chipped,
    Flawed,
    Normal,
    Flawless,
    Perfect,
}

impl GemQuality {
    #[allow(clippy::cast_sign_loss)]
    pub fn random_with_modifier(level: u32) -> Self {
        match level {
            0 => Self::Chipped,
            1 => match (fastrand::f32() * 100.) as u32 {
                0..=69 => Self::Chipped,
                _ => Self::Flawed,
            },
            2 => match (fastrand::f32() * 100.) as u32 {
                0..=59 => Self::Chipped,
                60..=89 => Self::Flawed,
                _ => Self::Normal,
            },
            3 => match (fastrand::f32() * 100.) as u32 {
                0..=49 => Self::Chipped,
                50..=79 => Self::Flawed,
                _ => Self::Normal,
            },
            4 => match (fastrand::f32() * 100.) as u32 {
                0..=39 => Self::Chipped,
                40..=69 => Self::Flawed,
                70..=89 => Self::Normal,
                _ => Self::Flawless,
            },
            5 => match (fastrand::f32() * 100.) as u32 {
                0..=29 => Self::Chipped,
                30..=59 => Self::Flawed,
                60..=89 => Self::Normal,
                _ => Self::Flawless,
            },
            6 => match (fastrand::f32() * 100.) as u32 {
                0..=19 => Self::Chipped,
                20..=49 => Self::Flawed,
                50..=79 => Self::Normal,
                _ => Self::Flawless,
            },
            7 => match (fastrand::f32() * 100.) as u32 {
                0..=9 => Self::Chipped,
                10..=39 => Self::Flawed,
                40..=69 => Self::Normal,
                _ => Self::Flawless,
            },
            _ => match (fastrand::f32() * 100.) as u32 {
                0..=29 => Self::Flawed,
                30..=59 => Self::Normal,
                60..=89 => Self::Flawless,
                _ => Self::Perfect,
            },
        }
    }
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
            GemType::Amethyst => Color::FUCHSIA,
            GemType::Opal => Color::ORANGE,
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

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tower {
    GemTower { typ: GemType, quality: GemQuality },
    Dirt,
}

impl Tower {
    pub fn add_abilities(self, entity: &mut EntityCommands) -> Entity {
        match self {
            Self::GemTower { typ, quality } => match (typ, quality) {
                (GemType::Emerald, GemQuality::Chipped) => entity.insert(SlowPoisonOnHit {
                    dps: 2,
                    slow: 15,
                    duration: 3.,
                }),
                (GemType::Emerald, GemQuality::Flawed) => entity.insert(SlowPoisonOnHit {
                    dps: 3,
                    slow: 20,
                    duration: 4.,
                }),
                (GemType::Emerald, GemQuality::Normal) => entity.insert(SlowPoisonOnHit {
                    dps: 5,
                    slow: 25,
                    duration: 5.,
                }),
                (GemType::Emerald, GemQuality::Flawless) => entity.insert(SlowPoisonOnHit {
                    dps: 8,
                    slow: 30,
                    duration: 6.,
                }),
                (GemType::Emerald, GemQuality::Perfect) => entity.insert(SlowPoisonOnHit {
                    dps: 16,
                    slow: 50,
                    duration: 8.,
                }),
                (GemType::Sapphire, quality) => entity.insert(SapphireSlowOnHit {
                    slow: match quality {
                        GemQuality::Chipped => 20,
                        GemQuality::Flawed => 25,
                        GemQuality::Normal => 30,
                        GemQuality::Flawless => 35,
                        GemQuality::Perfect => 40,
                    },
                }),
                (GemType::Diamond, ..) => entity.insert(CritOnHit),
                (GemType::Topaz, ..) => entity.insert(Target::Multiple(vec![])),
                (GemType::Ruby, GemQuality::Perfect) => entity.insert(SplashOnHit {
                    multiplier: 0.5,
                    range: 3.5,
                }),
                (GemType::Ruby, ..) => entity.insert(SplashOnHit {
                    multiplier: 0.5,
                    range: 3.,
                }),
                (GemType::Opal, GemQuality::Chipped) => entity.insert(Aura {
                    typ: AuraType::Opal(10),
                    range: 8.,
                }),
                (GemType::Opal, GemQuality::Flawed) => entity.insert(Aura {
                    typ: AuraType::Opal(15),
                    range: 9.,
                }),
                (GemType::Opal, GemQuality::Normal) => entity.insert(Aura {
                    typ: AuraType::Opal(20),
                    range: 10.,
                }),
                (GemType::Opal, GemQuality::Flawless) => entity.insert(Aura {
                    typ: AuraType::Opal(25),
                    range: 11.,
                }),
                (GemType::Opal, GemQuality::Perfect) => entity.insert(Aura {
                    typ: AuraType::Opal(35),
                    range: 12.,
                }),
                _ => entity,
            },
            Tower::Dirt => entity,
        }
        .id()
    }

    pub fn get_base_cooldown_time(self) -> f32 {
        match self {
            Tower::GemTower { typ, quality } => match (typ, quality) {
                (GemType::Aquamarine, _) => BASE_TOWER_SPEED / 2.,
                (
                    GemType::Topaz
                    | GemType::Amethyst
                    | GemType::Emerald
                    | GemType::Ruby
                    | GemType::Sapphire
                    | GemType::Diamond
                    | GemType::Opal,
                    GemQuality::Chipped,
                ) => BASE_TOWER_SPEED - 0.2,
                _ => BASE_TOWER_SPEED,
            },
            Tower::Dirt => BASE_TOWER_SPEED,
        }
    }

    pub fn get_refine(self) -> Self {
        match self {
            Tower::GemTower { typ, quality } => Self::GemTower {
                typ,
                quality: match quality {
                    GemQuality::Chipped => GemQuality::Flawed,
                    GemQuality::Flawed => GemQuality::Normal,
                    GemQuality::Normal => GemQuality::Flawless,
                    GemQuality::Flawless => GemQuality::Perfect,
                    GemQuality::Perfect => unimplemented!("Cannot refine perfect gems"),
                },
            },
            Tower::Dirt => self,
        }
    }

    pub fn get_y_offset(self) -> f32 {
        match self {
            Tower::GemTower { quality, .. } => match quality {
                GemQuality::Chipped => 0.2,
                GemQuality::Flawed => 0.4,
                GemQuality::Normal => 0.6,
                GemQuality::Flawless => 0.8,
                GemQuality::Perfect => 1.,
            },
            Tower::Dirt => 0.,
        }
    }
}

impl Display for Tower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tower::GemTower { typ, quality } => write!(
                f,
                "{}{}",
                match quality {
                    GemQuality::Chipped => "Chipped ",
                    GemQuality::Flawed => "Flawed ",
                    GemQuality::Normal => "",
                    GemQuality::Flawless => "Flawless ",
                    GemQuality::Perfect => "Perfect ",
                },
                match typ {
                    GemType::Emerald => "Emerald",
                    GemType::Ruby => "Ruby",
                    GemType::Sapphire => "Sapphire",
                    GemType::Diamond => "Diamond",
                    GemType::Amethyst => "Amethyst",
                    GemType::Opal => "Opal",
                    GemType::Aquamarine => "Aquamarine",
                    GemType::Topaz => "Topaz",
                }
            ),
            Tower::Dirt => write!(f, "Dirt"),
        }
    }
}

impl From<Tower> for LaserAttack {
    #[allow(clippy::match_same_arms)]
    fn from(value: Tower) -> Self {
        match value {
            Tower::GemTower { typ, quality } => Self {
                range: match (typ, quality) {
                    (GemType::Emerald, GemQuality::Chipped) => 5.,
                    (GemType::Emerald, GemQuality::Flawed) => 5.5,
                    (GemType::Emerald, GemQuality::Normal) => 6.,
                    (GemType::Emerald, GemQuality::Flawless) => 7.,
                    (GemType::Emerald, GemQuality::Perfect) => 7.,
                    (GemType::Ruby, _) => 8.,
                    (GemType::Sapphire, GemQuality::Chipped) => 5.5,
                    (GemType::Sapphire, GemQuality::Flawed) => 7.5,
                    (GemType::Sapphire, GemQuality::Normal) => 8.,
                    (GemType::Sapphire, GemQuality::Flawless) => 8.5,
                    (GemType::Sapphire, GemQuality::Perfect) => 14.,
                    (GemType::Diamond, GemQuality::Chipped) => 5.,
                    (GemType::Diamond, GemQuality::Flawed) => 5.5,
                    (GemType::Diamond, GemQuality::Normal) => 6.,
                    (GemType::Diamond, GemQuality::Flawless) => 6.5,
                    (GemType::Diamond, GemQuality::Perfect) => 7.5,
                    (GemType::Amethyst, GemQuality::Chipped) => 10.,
                    (GemType::Amethyst, GemQuality::Flawed) => 12.,
                    (GemType::Amethyst, GemQuality::Normal) => 13.,
                    (GemType::Amethyst, GemQuality::Flawless) => 13.5,
                    (GemType::Amethyst, GemQuality::Perfect) => 16.,
                    (GemType::Opal, GemQuality::Chipped) => 6.,
                    (GemType::Opal, GemQuality::Flawed) => 7.,
                    (GemType::Opal, GemQuality::Normal) => 8.,
                    (GemType::Opal, GemQuality::Flawless) => 9.,
                    (GemType::Opal, GemQuality::Perfect) => 10.,
                    (GemType::Aquamarine, GemQuality::Chipped) => 3.5,
                    (GemType::Aquamarine, GemQuality::Flawed) => 3.65,
                    (GemType::Aquamarine, GemQuality::Normal) => 3.8,
                    (GemType::Aquamarine, GemQuality::Flawless) => 4.,
                    (GemType::Aquamarine, GemQuality::Perfect) => 5.5,
                    (GemType::Topaz, _) => 5.,
                },
                color: typ.into(),
                damage: match (typ, quality) {
                    (GemType::Emerald, GemQuality::Chipped) => Damage::Range(4..=7),
                    (GemType::Emerald, GemQuality::Flawed) => Damage::Range(10..=13),
                    (GemType::Emerald, GemQuality::Normal) => Damage::Range(15..=25),
                    (GemType::Emerald, GemQuality::Flawless) => Damage::Range(30..=37),
                    (GemType::Emerald, GemQuality::Perfect) => Damage::Range(80..=95),
                    (GemType::Ruby, GemQuality::Chipped) => Damage::Range(8..=9),
                    (GemType::Ruby, GemQuality::Flawed) => Damage::Range(13..=16),
                    (GemType::Ruby, GemQuality::Normal) => Damage::Range(20..=25),
                    (GemType::Ruby, GemQuality::Flawless) => Damage::Range(38..=45),
                    (GemType::Ruby, GemQuality::Perfect) => Damage::Range(80..=100),
                    (GemType::Sapphire, GemQuality::Chipped) => Damage::Range(5..=8),
                    (GemType::Sapphire, GemQuality::Flawed) => Damage::Range(10..=14),
                    (GemType::Sapphire, GemQuality::Normal) => Damage::Range(16..=22),
                    (GemType::Sapphire, GemQuality::Flawless) => Damage::Range(30..=40),
                    (GemType::Sapphire, GemQuality::Perfect) => Damage::Range(60..=80),
                    (GemType::Diamond, GemQuality::Chipped) => Damage::Range(8..=12),
                    (GemType::Diamond, GemQuality::Flawed) => Damage::Range(16..=18),
                    (GemType::Diamond, GemQuality::Normal) => Damage::Range(30..=37),
                    (GemType::Diamond, GemQuality::Flawless) => Damage::Range(58..=65),
                    (GemType::Diamond, GemQuality::Perfect) => Damage::Range(140..=150),
                    (GemType::Amethyst, GemQuality::Chipped) => Damage::Range(10..=15),
                    (GemType::Amethyst, GemQuality::Flawed) => Damage::Range(20..=27),
                    (GemType::Amethyst, GemQuality::Normal) => Damage::Range(30..=45),
                    (GemType::Amethyst, GemQuality::Flawless) => Damage::Range(60..=80),
                    (GemType::Amethyst, GemQuality::Perfect) => Damage::Range(140..=170),
                    (GemType::Opal, GemQuality::Chipped) => Damage::Fixed(5),
                    (GemType::Opal, GemQuality::Flawed) => Damage::Fixed(10),
                    (GemType::Opal, GemQuality::Normal) => Damage::Fixed(20),
                    (GemType::Opal, GemQuality::Flawless) => Damage::Fixed(40),
                    (GemType::Opal, GemQuality::Perfect) => Damage::Fixed(85),
                    (GemType::Aquamarine, GemQuality::Chipped) => Damage::Range(6..=8),
                    (GemType::Aquamarine, GemQuality::Flawed) => Damage::Range(12..=15),
                    (GemType::Aquamarine, GemQuality::Normal) => Damage::Range(24..=30),
                    (GemType::Aquamarine, GemQuality::Flawless) => Damage::Range(48..=55),
                    (GemType::Aquamarine, GemQuality::Perfect) => Damage::Range(100..=120),
                    (GemType::Topaz, GemQuality::Chipped) => Damage::Fixed(4),
                    (GemType::Topaz, GemQuality::Flawed) => Damage::Fixed(8),
                    (GemType::Topaz, GemQuality::Normal) => Damage::Fixed(14),
                    (GemType::Topaz, GemQuality::Flawless) => Damage::Fixed(25),
                    (GemType::Topaz, GemQuality::Perfect) => Damage::Fixed(75),
                },
                hits: match typ {
                    GemType::Diamond => Hits::Ground,
                    GemType::Amethyst => Hits::Flying,
                    _ => Hits::All,
                },
            },
            Tower::Dirt => unimplemented!(),
        }
    }
}

impl From<Tower> for Cooldown {
    fn from(value: Tower) -> Self {
        // Make a cooldown timer that starts in a finished state
        let time = value.get_base_cooldown_time();
        let mut timer = Timer::from_seconds(time, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(time));

        Cooldown(timer)
    }
}

impl From<Tower> for Mesh {
    fn from(value: Tower) -> Self {
        match value {
            Tower::GemTower { quality, .. } => shape::Cube {
                size: match quality {
                    GemQuality::Chipped => 0.4,
                    GemQuality::Flawed => 0.8,
                    GemQuality::Normal => 1.2,
                    GemQuality::Flawless => 1.6,
                    GemQuality::Perfect => 2.0,
                },
            }
            .into(),
            Tower::Dirt => shape::Box {
                min_x: -1.,
                max_x: 1.,
                min_y: 0.,
                max_y: 0.2,
                min_z: -1.,
                max_z: 1.,
            }
            .into(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Hits {
    Ground,
    Flying,
    All,
}

impl Display for Hits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Hits::Ground => "Ground only",
                Hits::Flying => "Air only",
                Hits::All => "Everything",
            }
        )
    }
}

#[derive(Component)]
pub struct LaserAttack {
    pub range: f32,
    pub color: Color,
    pub damage: Damage,
    pub hits: Hits,
}

impl LaserAttack {
    pub fn attack(
        mut lines: ResMut<DebugLines>,
        time: Res<Time>,
        mut writer: EventWriter<Hit>,
        mut towers: Query<
            (
                Entity,
                &mut Cooldown,
                &mut Target,
                &GlobalTransform,
                &LaserAttack,
            ),
            With<Tower>,
        >,
        positions: Query<(Entity, &Transform, &Creep), With<HitPoints>>,
    ) {
        for (tower, mut cooldown, mut target, tower_pos, attack) in &mut towers {
            cooldown.tick(time.delta());
            if cooldown.finished() {
                let targets = match target.clone() {
                    Target::Single(target) => match target {
                        Some(target) => vec![target],
                        None => vec![],
                    },
                    Target::Multiple(targets) => targets,
                };
                for target_entity in &targets {
                    // Tower has a target
                    if let Ok((_, target_pos, _)) = positions.get(*target_entity) {
                        if target_pos
                            .translation
                            .distance_squared(tower_pos.translation())
                            > attack.range.powf(2.)
                        {
                            // Target is out of range
                            match target.clone() {
                                Target::Single(_) => *target = Target::Single(None),
                                Target::Multiple(targets) => {
                                    *target = Target::Multiple(
                                        targets
                                            .iter()
                                            .filter(|targ| target_entity != *targ)
                                            .copied()
                                            .collect(),
                                    );
                                }
                            }
                        } else {
                            // Target is alive and in range
                            cooldown.reset();
                            lines.line_colored(
                                tower_pos.translation(),
                                target_pos.translation,
                                0.25,
                                attack.color,
                            );

                            writer.send(Hit {
                                source: tower,
                                target: *target_entity,
                                value: attack.damage.clone().get_value(),
                            });
                        }
                    } else {
                        // Target is dead
                        match target.clone() {
                            Target::Single(_) => *target = Target::Single(None),
                            Target::Multiple(targets) => {
                                *target = Target::Multiple(
                                    targets
                                        .iter()
                                        .filter(|targ| target_entity != *targ)
                                        .copied()
                                        .collect(),
                                );
                            }
                        }
                    }
                }
                if targets.is_empty() {
                    // Tower needs to find a new target
                    if let Target::Single(_) = *target {
                        let closest = Self::get_closest_creep(
                            positions.iter().map(|(entity, position, Creep { typ })| {
                                (entity, position.translation, *typ)
                            }),
                            tower_pos.translation(),
                        );

                        if let Some((creep, distance, typ)) = closest {
                            if attack.range.powf(2.) >= distance && typ.hits(attack.hits) {
                                *target = Target::Single(Some(creep));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_multiple_targets(
        positions: Query<(Entity, &Transform, &Creep), (With<HitPoints>, Changed<Transform>)>,
        mut towers: Query<(&mut Target, &GlobalTransform, &LaserAttack), With<Tower>>,
    ) {
        for (mut target, tower_pos, attack) in &mut towers {
            if let Target::Multiple(_) = target.clone() {
                let targets = Self::get_creeps_in_range(
                    positions.iter().map(|(entity, position, Creep { typ })| {
                        (entity, position.translation, *typ)
                    }),
                    tower_pos.translation(),
                    attack.range,
                )
                .iter()
                .filter(|(_, typ)| typ.hits(attack.hits))
                .map(|(entity, _)| *entity)
                .collect();
                *target = Target::Multiple(targets);
            }
        }
    }

    fn get_closest_creep(
        creeps: impl Iterator<Item = (Entity, Vec3, CreepType)>,
        position: Vec3,
    ) -> Option<(Entity, f32, CreepType)> {
        let mut closest = None;
        let mut closest_distance_squared = f32::MAX;
        for (creep, creep_pos, typ) in creeps {
            let distance_squared = creep_pos.distance_squared(position);
            if distance_squared < closest_distance_squared {
                closest = Some((creep, distance_squared, typ));
                closest_distance_squared = distance_squared;
            }
        }
        closest
    }

    fn get_creeps_in_range(
        creeps: impl Iterator<Item = (Entity, Vec3, CreepType)>,
        position: Vec3,
        range: f32,
    ) -> Vec<(Entity, CreepType)> {
        creeps
            .filter(|(_, creep_pos, _)| creep_pos.distance_squared(position) <= range.powf(2.))
            .map(|(creep, _, typ)| (creep, typ))
            .collect()
    }
}

#[derive(Clone)]
pub enum Damage {
    Range(RangeInclusive<u32>),
    Fixed(u32),
}

impl Damage {
    pub fn get_value(self) -> u32 {
        match self {
            Damage::Range(range) => fastrand::u32(range),
            Damage::Fixed(val) => val,
        }
    }
}

impl Display for Damage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Range(range) => format!(
                    "{}-{}",
                    range.clone().min().unwrap(),
                    range.clone().max().unwrap()
                ),
                Self::Fixed(fixed) => format!("{fixed}"),
            }
        )
    }
}

#[derive(Default)]
pub struct PickSelectedTower;

impl PickSelectedTower {
    pub fn pick_building(
        mut commands: Commands,
        mut events: EventReader<PickSelectedTower>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut next_phase: ResMut<NextState<Phase>>,
        mut selected: Option<ResMut<SelectedTower>>,
        just_built: Query<(Entity, &GlobalTransform), With<JustBuilt>>,
    ) {
        for _ in events.iter() {
            if let Some(SelectedTower {
                tower, pickable, ..
            }) = selected.as_deref_mut()
            {
                if !*pickable {
                    continue;
                }
                for (entity, transform) in &just_built {
                    if entity == *tower {
                        commands.entity(entity).remove::<JustBuilt>();
                    } else {
                        commands.entity(entity).despawn_recursive();
                        commands.spawn((
                            PbrBundle {
                                mesh: meshes.add(Tower::Dirt.into()),
                                material: mats.add(Color::ORANGE_RED.into()),
                                transform: Transform::from_xyz(
                                    transform.compute_transform().translation.x,
                                    Tower::Dirt.get_y_offset(),
                                    transform.compute_transform().translation.z,
                                ),
                                ..default()
                            },
                            Name::new("Dirt"),
                            Tower::Dirt,
                        ));
                    }
                }
                *pickable = false;
                next_phase.set(Phase::Spawn);
            }
        }
    }
}

#[derive(Default)]
pub struct RemoveSelectedTower;

impl RemoveSelectedTower {
    pub fn remove(
        mut commands: Commands,
        mut events: EventReader<RemoveSelectedTower>,
        mut build_grid: ResMut<BuildGrid>,
        selected: Option<Res<SelectedTower>>,
        towers: Query<(&Tower, &GlobalTransform), Without<JustBuilt>>,
    ) {
        for _ in events.iter() {
            if let Some(SelectedTower {
                tower, removable, ..
            }) = selected.as_deref()
            {
                if !removable {
                    continue;
                }
                if let Ok((Tower::Dirt, transform)) = towers.get(*tower) {
                    commands.entity(*tower).despawn_recursive();

                    #[allow(clippy::cast_sign_loss)]
                    let positions = get_squares_from_pos(transform.translation().xz())
                        .map(|pos| UVec2::new((pos.x - 0.5) as u32, (pos.y - 0.5) as u32));
                    for pos in positions {
                        build_grid.remove(&pos);
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct JustBuilt;

#[derive(Default)]
pub struct RefineAndPickSelectedTower;

impl RefineAndPickSelectedTower {
    pub fn refine_and_pick(
        mut commands: Commands,
        mut refine_events: EventReader<RefineAndPickSelectedTower>,
        mut pick_events: EventWriter<PickSelectedTower>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        selected: Option<Res<SelectedTower>>,
        towers: Query<(Entity, &GlobalTransform, &Tower)>,
    ) {
        for _ in refine_events.iter() {
            if let Some(SelectedTower {
                tower, refinable, ..
            }) = selected.as_deref()
            {
                if !refinable {
                    continue;
                }
                if let Ok((old_tower, tower_pos, tower)) = towers.get(*tower) {
                    commands.entity(old_tower).despawn_recursive();
                    match tower.get_refine() {
                        Tower::GemTower { typ, quality } => {
                            let new_tower = Tower::GemTower { typ, quality };
                            let cooldown: Cooldown = new_tower.into();
                            let new_tower = new_tower.add_abilities(&mut commands.spawn((
                                PbrBundle {
                                    mesh: meshes.add(new_tower.into()),
                                    material: mats.add(typ.into()),
                                    transform: Transform::from_xyz(
                                        tower_pos.compute_transform().translation.x,
                                        new_tower.get_y_offset(),
                                        tower_pos.compute_transform().translation.z,
                                    ),
                                    ..default()
                                },
                                Name::new(new_tower.to_string()),
                                new_tower,
                                LaserAttack::from(new_tower),
                                cooldown,
                                Target::Single(None),
                                SpeedModifiers::default(),
                                JustBuilt,
                            )));
                            commands.insert_resource(SelectedTower {
                                tower: new_tower,
                                pickable: true,
                                refinable: false,
                                removable: false,
                            });
                            pick_events.send(PickSelectedTower);
                        }
                        Tower::Dirt => println!("Can't refine and pick dirt"),
                    }
                }
            }
        }
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct RandomLevel(u32);
