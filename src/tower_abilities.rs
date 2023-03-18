use bevy::prelude::*;

use crate::{
    creeps::{Dead, Hit, HitPoints, Slow, SlowSource},
    Phase,
};

pub struct TowerAbilitiesPlugin;

impl Plugin for TowerAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            SlowPoisonOnHit::on_hit.in_set(OnUpdate(Phase::Spawn)),
            SlowPoison::add.in_set(OnUpdate(Phase::Spawn)),
            SlowPoison::update.in_set(OnUpdate(Phase::Spawn)),
        ));
    }
}

#[derive(Component, Clone, Copy)]
pub struct SlowPoisonOnHit {
    pub dps: u32,
    pub slow: u32,
    pub duration: f32,
}

#[derive(Component)]
pub struct SlowPoison {
    dps: u32,
    slow: u32,
    duration: Timer,
}

impl From<SlowPoisonOnHit> for SlowPoison {
    fn from(value: SlowPoisonOnHit) -> Self {
        Self {
            dps: value.dps,
            slow: value.slow,
            duration: Timer::from_seconds(value.duration, TimerMode::Once),
        }
    }
}

impl SlowPoisonOnHit {
    pub fn on_hit(
        mut commands: Commands,
        mut events: EventReader<Hit>,
        towers: Query<&SlowPoisonOnHit>,
    ) {
        for Hit { source, target, .. } in events.iter() {
            if let Ok(on_hit) = towers.get(*source) {
                commands
                    .entity(*target)
                    .insert(Into::<SlowPoison>::into(*on_hit));
            }
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct PoisonTimer(Timer);
impl Default for PoisonTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}

impl SlowPoison {
    pub fn add(mut creeps: Query<(&SlowPoison, &mut Slow), Changed<SlowPoison>>) {
        for (poison, mut slow) in &mut creeps {
            slow.insert(SlowSource::Poison, poison.slow);
        }
    }
    pub fn update(
        mut commands: Commands,
        mut events: EventWriter<Dead>,
        mut poison_timer: Local<PoisonTimer>,
        time: Res<Time>,
        mut creeps: Query<(Entity, &mut HitPoints, &mut SlowPoison, &mut Slow)>,
    ) {
        poison_timer.tick(time.delta());
        for (creep, mut hitpoints, mut poison, mut slow) in &mut creeps {
            if poison_timer.just_finished() {
                hitpoints.sub(poison.dps);
                if hitpoints.dead() {
                    events.send(Dead(creep));
                }
            }
            if poison.duration.tick(time.delta()).finished() {
                commands.entity(creep).remove::<SlowPoison>();
                slow.remove(&SlowSource::Poison);
            }
        }
    }
}
