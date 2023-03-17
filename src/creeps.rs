use bevy::prelude::{shape::Cube, *};
use seldom_map_nav::prelude::*;

use crate::{
    common::{CreepPos, TrackWorldObjectToScreenPosition},
    progress_bar::ProgressBar,
    towers::Hits,
    CurrentLevel, Phase, CREEP_CLEARANCE, MAP_WIDTH, RESOLUTION, WINDOW_HEIGHT,
};

#[derive(Component)]
pub struct Creep {
    pub typ: CreepType,
}

pub struct Damaged {
    pub target: Entity,
    pub value: u32,
}

impl Damaged {
    pub fn consume(
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
pub struct HitPoints {
    max: u32,
    current: u32,
}

impl HitPoints {
    pub fn new(value: u32) -> Self {
        Self {
            max: value,
            current: value,
        }
    }

    fn sub(&mut self, value: u32) {
        self.current = self.current.saturating_sub(value);
    }

    fn dead(&self) -> bool {
        self.current == 0
    }

    fn ratio(&self) -> f32 {
        self.current as f32 / self.max as f32
    }

    pub fn spawn_health_bars(
        mut commands: Commands,
        hitpoints: Query<(Entity, &HitPoints), Added<HitPoints>>,
    ) {
        for (entity, hitpoints) in &hitpoints {
            let bar = ProgressBar::spawn(
                (WINDOW_HEIGHT * RESOLUTION / 2.0, WINDOW_HEIGHT / 2.0).into(),
                Color::GREEN,
                Color::RED,
                hitpoints.ratio(),
                entity,
                &mut commands,
            );

            commands
                .entity(bar)
                .insert(TrackWorldObjectToScreenPosition {
                    target: entity,
                    offset: Vec2::new(0.0, 21.0),
                });

            commands.entity(entity).insert(UpdateHitpointsBar(bar));
        }
    }

    pub fn update_health_bars(
        mut health_bars: Query<(&mut Style, &ProgressBar)>,
        hitpoints: Query<&HitPoints, Changed<HitPoints>>,
    ) {
        for (mut style, bar) in &mut health_bars {
            if let Ok(hitpoints) = hitpoints.get(bar.target) {
                style.size.width = Val::Percent(hitpoints.ratio() * 100.0);
            }
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn from_level(level: u32) -> Self {
        Self::new(match level {
            1 => 10,
            2 => 30,
            3 => 55,
            4 => 70,
            5 => 90,
            6 => 120,
            7 => 178,
            8 => 240,
            9 => 300,
            10 => 470,
            11 => 490,
            12 => 450,
            13 => 570,
            14 => 650,
            15 => 1_000,
            16 => 725,
            17 => 1_350,
            18 => 1_550,
            19 => 1_950,
            20 => 1_350,
            21 => 2_300,
            22 => 2_530,
            23 => 3_000,
            24 => 2_500,
            25 => 3_750,
            26 => 4_500,
            27 => 5_000,
            28 => 4_150,
            29 => 6_750,
            30 => 7_150,
            31 => 8_000,
            32 => 6_200,
            33 => 9_550,
            34 => 10_200,
            35 => 11_500,
            36 => 8_500,
            37 => 13_000,
            38 => 15_000,
            39 => 17_000,
            40 => 10_500,
            41 => 19_500,
            _ => 23_000,
        })
    }
}

#[derive(Component)]
struct UpdateHitpointsBar(Entity);

#[derive(Deref, DerefMut)]
pub struct Dead(pub Entity);

impl Dead {
    pub fn death(
        mut commands: Commands,
        mut reader: EventReader<Dead>,
        bars: Query<(&ProgressBar, &Parent)>,
    ) {
        for Dead(dead) in reader.iter() {
            if let Some(entity) = commands.get_entity(*dead) {
                entity.despawn_recursive();
                for (bar, parent) in bars.iter() {
                    if bar.target == *dead {
                        commands.entity(**parent).despawn_recursive();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CreepSpawner {
    pub timer: Timer,
    pub amount: u32,
}

impl Default for CreepSpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Repeating),
            amount: 20,
        }
    }
}

impl CreepSpawner {
    pub fn spawn(
        mut commands: Commands,
        mut phase: ResMut<NextState<Phase>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        time: Res<Time>,
        level: Res<CurrentLevel>,
        mut spawners: Query<&mut CreepSpawner>,
        creeps: Query<(), With<Creep>>,
        navmeshes: Query<Entity, With<Navmeshes>>,
    ) {
        let mut spawns_left = 0;
        for mut spawner in &mut spawners {
            if spawner.amount == 0 {
                continue;
            }
            if !spawner.timer.tick(time.delta()).just_finished() {
                spawns_left += spawner.amount;
                continue;
            }
            spawner.amount = spawner.amount.saturating_sub(1);
            spawns_left += spawner.amount;
            let navmesh = navmeshes.single();
            let typ = CreepType::from_level(**level);
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cube { size: 0.5 }.into()),
                    material: mats.add(Color::BLACK.into()),
                    transform: Transform::from_xyz(
                        0.5,
                        match typ {
                            CreepType::Ground => 0.25,
                            CreepType::Flying => 1.25,
                        },
                        MAP_WIDTH as f32 - 1.,
                    ),
                    ..default()
                },
                Creep { typ },
                HitPoints::from_level(**level),
                NavBundle {
                    pathfind: Pathfind::new(
                        navmesh,
                        CREEP_CLEARANCE,
                        None,
                        PathTarget::Static(Vec2::new(0.5 + MAP_WIDTH as f32 - 1., 0.5)),
                        NavQuery::Accuracy,
                        NavPathMode::Accuracy,
                    ),
                    nav: Nav::new(1.),
                },
                CreepPos {
                    pos: Vec2::new(0.5, MAP_WIDTH as f32 - 1.),
                },
                Name::new("Creep"),
            ));
        }
        if spawns_left == 0 && creeps.iter().count() == 0 && phase.0.is_none() {
            phase.set(Phase::Build);
        }
    }

    pub fn reset_amount_system(mut spawners: Query<&mut CreepSpawner>) {
        for mut spawner in &mut spawners {
            spawner.amount = CreepSpawner::default().amount;
        }
    }
}

#[derive(Clone, Copy)]
pub enum CreepType {
    Ground,
    Flying,
}

impl CreepType {
    pub fn from_level(level: u32) -> Self {
        if level % 4 == 0 {
            CreepType::Flying
        } else {
            CreepType::Ground
        }
    }

    pub fn hits(self, hits: Hits) -> bool {
        match hits {
            Hits::Ground => matches!(self, CreepType::Ground),
            Hits::Flying => matches!(self, CreepType::Flying),
            Hits::All => true,
        }
    }
}
