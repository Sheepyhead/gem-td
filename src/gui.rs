use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    controls::SelectedTower,
    towers::{Dirt, JustBuilt, PickTower, RemoveTower},
    Phase,
};

pub fn show_sidebar(
    mut contexts: EguiContexts,
    mut pick_events: EventWriter<PickTower>,
    mut remove_events: EventWriter<RemoveTower>,
    phase: Res<State<Phase>>,
    selected: Res<SelectedTower>,
    towers: Query<&Name>,
    dirt: Query<(), With<Dirt>>,
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
                    if let Ok(name) = towers.get(selected_tower) {
                        ui.label(format!("Selected tower: {name}"));
                    }
                    match phase.0 {
                        Phase::Pick => {
                            if just_built.contains(selected_tower)
                                && ui.button("Pick this tower").clicked()
                            {
                                pick_events.send(PickTower(selected_tower));
                            };
                            if dirt.contains(selected_tower) && ui.button("Remove").clicked() {
                                remove_events.send(RemoveTower(selected_tower));
                            }
                        }
                        Phase::Build => {
                            if dirt.contains(selected_tower) && ui.button("Remove").clicked() {
                                remove_events.send(RemoveTower(selected_tower));
                            }
                        }
                        Phase::Spawn => {}
                    }
                }
            });
        });
}
