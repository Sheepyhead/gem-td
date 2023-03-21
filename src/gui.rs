use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    controls::SelectedTower,
    towers::{
        Cooldown, JustBuilt, LaserAttack, PickTower, RandomLevel, RemoveTower, Tower, Upgrade,
        UpgradeAndPick,
    },
    Phase,
};

pub fn show_sidebar(
    mut contexts: EguiContexts,
    mut pick_events: EventWriter<PickTower>,
    mut remove_events: EventWriter<RemoveTower>,
    mut upgrade_and_pick_events: EventWriter<UpgradeAndPick>,
    mut upgrade_events: EventWriter<Upgrade>,
    mut chance: ResMut<RandomLevel>,
    phase: Res<State<Phase>>,
    selected: Res<SelectedTower>,
    names: Query<&Name>,
    tower_stats: Query<(&LaserAttack, &Cooldown)>,
    just_built_towers: Query<&Tower, With<JustBuilt>>,
    towers: Query<&Tower, Without<JustBuilt>>,
    just_built: Query<(), With<JustBuilt>>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(match phase.0 {
                    Phase::Build => "Dig five holes!",
                    Phase::Pick => "Pick which gem to keep!",
                    // TODO: add logic to calculate monsters left
                    Phase::Spawn => "Monsters left: some",
                });

                ui.separator();

                if let Some(selected_tower) = **selected {
                    if let Ok(name) = names.get(selected_tower) {
                        ui.label(format!("Selected tower: {name}"));
                    }

                    if let Ok((
                        LaserAttack {
                            range,
                            damage,
                            hits,
                            ..
                        },
                        Cooldown(timer),
                    )) = tower_stats.get(selected_tower)
                    {
                        ui.label(format!("Range: {range}"));
                        ui.label(format!("Damage: {damage}"));
                        ui.label(format!("Targets: {hits}"));
                        ui.label(format!("Attack speed: {}", timer.duration().as_secs_f32()));
                    }
                    match phase.0 {
                        Phase::Pick => {
                            if just_built.contains(selected_tower)
                                && ui.button("Pick this tower").clicked()
                            {
                                pick_events.send(PickTower(selected_tower));
                            };
                            if let Ok(Tower::Dirt) = towers.get(selected_tower) {
                                if ui.button("Remove").clicked() {
                                    remove_events.send(RemoveTower(selected_tower));
                                }
                            }
                            if let Ok(Tower::GemTower { typ, quality }) =
                                just_built_towers.get(selected_tower)
                            {
                                if just_built_towers
                                    .iter()
                                    .filter(|other_gem_tower| {
                                        if let Tower::GemTower { .. } = other_gem_tower {
                                            **other_gem_tower
                                                == Tower::GemTower {
                                                    typ: *typ,
                                                    quality: *quality,
                                                }
                                        } else {
                                            false
                                        }
                                    })
                                    .count()
                                    >= 2
                                    && ui.button("Combine!").clicked()
                                {
                                    upgrade_and_pick_events.send(UpgradeAndPick(selected_tower));
                                }
                            }
                        }
                        Phase::Build => {
                            if let Ok(Tower::Dirt) = towers.get(selected_tower) {
                                if ui.button("Remove").clicked() {
                                    remove_events.send(RemoveTower(selected_tower));
                                }
                            }
                        }
                        Phase::Spawn => {
                            if let Ok(tower) = towers.get(selected_tower) {
                                if towers
                                    .iter()
                                    .filter(|other_gem_tower| {
                                        if let Tower::GemTower { .. } = other_gem_tower {
                                            *other_gem_tower == tower
                                        } else {
                                            false
                                        }
                                    })
                                    .count()
                                    >= 2
                                    && ui.button("Combine!").clicked()
                                {
                                    upgrade_events.send(Upgrade(selected_tower));
                                }
                            }
                        }
                    }
                }

                if ui
                    .button(format!("Upgrade chance to {}", **chance + 1))
                    .clicked()
                {
                    **chance += 1;
                }
            });
        });
}
