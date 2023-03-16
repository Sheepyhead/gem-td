use bevy::prelude::*;
use seldom_map_nav::prelude::*;

use crate::{
    common::TrackWorldObjectToScreenPosition, progress_bar::ProgressBar, Phase, CREEP_CLEARANCE,
    RESOLUTION, WINDOW_HEIGHT,
};

#[derive(Component)]
pub struct Creep;

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
            amount: 10,
        }
    }
}

impl CreepSpawner {
    pub fn spawn(
        mut commands: Commands,
        mut phase: ResMut<NextState<Phase>>,
        ass: Res<AssetServer>,
        time: Res<Time>,
        mut spawners: Query<(&GlobalTransform, &mut CreepSpawner)>,
        creeps: Query<(), With<Creep>>,
        navmeshes: Query<Entity, With<Navmeshes>>,
    ) {
        let mut spawns_left = 0;
        for (transform, mut spawner) in &mut spawners {
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
            commands.spawn((
                SpriteBundle {
                    texture: ass.load("creep.png"),
                    transform: transform.compute_transform(),
                    ..default()
                },
                Creep,
                HitPoints::new(500),
                NavBundle {
                    pathfind: Pathfind::new(
                        navmesh,
                        CREEP_CLEARANCE,
                        None,
                        PathTarget::Static(Vec2::new(0., 15.)),
                        NavQuery::Accuracy,
                        NavPathMode::Accuracy,
                    ),
                    nav: Nav::new(100.0),
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
