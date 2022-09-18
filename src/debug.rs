use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

pub struct Debug;

impl Plugin for Debug {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WorldInspectorParams {
            enabled: false,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(toggle_inspector);
    }
}

fn toggle_inspector(
    input: ResMut<Input<KeyCode>>,
    mut window_params: ResMut<WorldInspectorParams>,
) {
    if input.just_pressed(KeyCode::Grave) {
        window_params.enabled = !window_params.enabled;
    }
}
