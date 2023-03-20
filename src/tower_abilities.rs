use std::time::Duration;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    creeps::{Dead, Hit, HitPoints, Slow, SlowSource},
    towers::{Cooldown, GemTower, Tower},
    Phase,
};

pub struct TowerAbilitiesPlugin;

impl Plugin for TowerAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            SlowPoisonOnHit::on_hit.in_set(OnUpdate(Phase::Spawn)),
            SlowPoison::add.in_set(OnUpdate(Phase::Spawn)),
            SlowPoison::update.in_set(OnUpdate(Phase::Spawn)),
            SapphireSlowOnHit::on_hit.in_set(OnUpdate(Phase::Spawn)),
            SapphireSlow::changed.in_set(OnUpdate(Phase::Spawn)),
            SapphireSlow::update.in_set(OnUpdate(Phase::Spawn)),
            CritOnHit::crit.in_set(OnUpdate(Phase::Spawn)),
            SplashOnHit::splash.in_set(OnUpdate(Phase::Spawn)),
            Aura::aura_tower_added,
            Aura::aura_tower_removed,
            Aura::tower_added,
            SpeedModifiers::update,
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

#[derive(Component, Clone, Copy)]
pub struct SapphireSlowOnHit {
    pub slow: u32,
}

impl SapphireSlowOnHit {
    pub fn on_hit(
        mut commands: Commands,
        mut events: EventReader<Hit>,
        towers: Query<&SapphireSlowOnHit>,
    ) {
        for Hit { source, target, .. } in events.iter() {
            if let Ok(tower) = towers.get(*source) {
                commands
                    .entity(*target)
                    .insert(Into::<SapphireSlow>::into(*tower));
            }
        }
    }
}

#[derive(Component)]
struct SapphireSlow {
    slow: u32,
    duration: Timer,
}

impl From<SapphireSlowOnHit> for SapphireSlow {
    fn from(value: SapphireSlowOnHit) -> Self {
        Self {
            slow: value.slow,
            duration: Timer::from_seconds(4., TimerMode::Once),
        }
    }
}

impl SapphireSlow {
    fn changed(mut creeps: Query<(&mut Slow, &SapphireSlow), Changed<SapphireSlow>>) {
        for (mut slow, sapphire) in &mut creeps {
            slow.insert(SlowSource::Sapphire, sapphire.slow);
        }
    }
    fn update(
        mut commands: Commands,
        time: Res<Time>,
        mut creeps: Query<(Entity, &mut Slow, &mut SapphireSlow)>,
    ) {
        for (entity, mut slow, mut sapphire) in &mut creeps {
            if sapphire.duration.tick(time.delta()).finished() {
                commands.entity(entity).remove::<SapphireSlow>();
                slow.remove(&SlowSource::Sapphire);
            }
        }
    }
}

#[derive(Component)]
pub struct CritOnHit;

impl CritOnHit {
    fn crit(
        mut hits: EventReader<Hit>,
        mut deads: EventWriter<Dead>,
        mut creeps: Query<&mut HitPoints>,
        towers: Query<(), With<CritOnHit>>,
    ) {
        for Hit {
            source,
            target,
            value,
        } in hits.iter()
        {
            if let (Ok(..), Ok(mut creep)) = (towers.get(*source), creeps.get_mut(*target)) {
                if fastrand::f32() < 0.25 {
                    println!("CRIT!");
                    creep.sub(*value);
                    if creep.dead() {
                        deads.send(Dead(*target));
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SplashOnHit {
    pub multiplier: f32,
    pub range: f32,
}

impl SplashOnHit {
    fn splash(
        mut hits: EventReader<Hit>,
        mut dead: EventWriter<Dead>,
        towers: Query<&SplashOnHit>,
        mut creeps: Query<(Entity, &GlobalTransform, &mut HitPoints)>,
    ) {
        for Hit {
            source,
            target,
            value,
        } in hits.iter()
        {
            if let Ok(SplashOnHit { multiplier, range }) = towers.get(*source) {
                let target_pos = match creeps.get(*target) {
                    Ok(value) => *value.1,
                    Err(_) => continue,
                };
                for (creep, _, mut hitpoints) in
                    creeps.iter_mut().filter(|(creep, transform, _)| {
                        creep != target
                            && transform
                                .translation()
                                .distance_squared(target_pos.translation())
                                <= range.powf(2.)
                    })
                {
                    #[allow(clippy::cast_sign_loss)]
                    hitpoints.sub((*value as f32 * multiplier) as u32);
                    if hitpoints.dead() {
                        dead.send(Dead(creep));
                    }
                }
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum AuraType {
    Opal(u32),
}

#[derive(Component)]
pub struct Aura {
    pub typ: AuraType,
    pub range: f32,
}

impl Aura {
    fn aura_tower_added(
        aura_changed: Query<(), Changed<Aura>>,
        auras: Query<(&GlobalTransform, &Aura)>,
        mut towers: Query<(Entity, &GlobalTransform, &mut SpeedModifiers)>,
    ) {
        if !aura_changed.is_empty() {
            // New aura tower has been added, recalculate
            Self::reapply_auras(&mut towers, auras.iter());
            println!("New aura tower!");
        }
    }

    fn aura_tower_removed(
        removed: RemovedComponents<Aura>,
        auras: Query<(&GlobalTransform, &Aura)>,
        mut towers: Query<(Entity, &GlobalTransform, &mut SpeedModifiers)>,
    ) {
        if !removed.is_empty() {
            // Existing aura tower has been removed, recalculate
            Self::reapply_auras(&mut towers, auras.iter());
            println!("Removed aura tower!");
        }
    }

    fn tower_added(
        tower_added: Query<(), (Added<Tower>, Without<Aura>)>,
        auras: Query<(&GlobalTransform, &Aura)>,
        mut towers: Query<(Entity, &GlobalTransform, &mut SpeedModifiers)>,
    ) {
        if !tower_added.is_empty() {
            // New tower has been added, recalculate
            Self::reapply_auras(&mut towers, auras.iter());
            println!("New non-aura tower!");
        }
    }

    fn reapply_auras<'a>(
        towers: &mut Query<(Entity, &GlobalTransform, &mut SpeedModifiers)>,
        auras: impl Iterator<Item = (&'a GlobalTransform, &'a Aura)>,
    ) {
        let mut strongest_auras: HashSet<(Entity, AuraType)> = HashSet::default();
        for (aura_pos, aura) in auras {
            for (tower, tower_pos, _) in towers.iter() {
                if aura_pos
                    .translation()
                    .distance_squared(tower_pos.translation())
                    <= aura.range.powf(2.)
                {
                    // Tower is within range of aura
                    if let Some((entity, existing_aura)) = strongest_auras.get(&(tower, aura.typ)) {
                        // Tower is already affected by an aura of this type
                        #[allow(irrefutable_let_patterns)]
                        if let (AuraType::Opal(existing_value), AuraType::Opal(new_value)) =
                            (existing_aura, aura.typ)
                        {
                            // Test if the new value is bigger than the old
                            if *existing_value < new_value {
                                strongest_auras.insert((*entity, AuraType::Opal(new_value)));
                            }
                        }
                    } else {
                        // Tower is not yet affected by an aura of this type
                        strongest_auras.insert((tower, aura.typ));
                    }
                }
            }
        }
        // Apply auras
        for (tower, aura) in strongest_auras {
            match aura {
                AuraType::Opal(modifier) => {
                    if let Ok((_, _, mut modifiers)) = towers.get_mut(tower) {
                        modifiers.insert(SpeedModifierType::OpalAura, modifier);
                    }
                }
            }
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct SpeedModifiers(HashMap<SpeedModifierType, u32>);

#[derive(Eq, Hash, PartialEq)]
pub enum SpeedModifierType {
    OpalAura,
}

impl SpeedModifiers {
    fn update(
        mut modifiers: Query<(&GemTower, &mut Cooldown, &SpeedModifiers), Changed<SpeedModifiers>>,
    ) {
        for (tower, mut cooldown, modifiers) in &mut modifiers {
            println!("Speed modifier changed for tower {tower:?}");
            // Make a cooldown timer that starts in a finished state
            let base_time = tower.get_base_cooldown_time();
            let mut time = base_time;
            for modifier in modifiers.values() {
                time -= base_time * (*modifier as f32 / 100.);
            }
            let mut timer = Timer::from_seconds(time, TimerMode::Once);
            timer.tick(Duration::from_secs_f32(time));
            *cooldown = Cooldown(timer);
        }
    }
}
