use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{GameState, Player, UnderCursor};

pub struct Input;

impl Plugin for Input {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerActions>::default())
            .add_enter_system_set(
                GameState::InGame,
                ConditionSet::new().with_system(setup_player_input).into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(build_at_cursor)
                    .into(),
            );
    }
}

#[derive(Actionlike, Clone)]
enum PlayerActions {
    BuildAtCursor,
}

fn setup_player_input(mut commands: Commands) {
    commands.spawn((
        Player { number: 0 },
        InputManagerBundle::<PlayerActions> {
            input_map: InputMap::new([(KeyCode::B, PlayerActions::BuildAtCursor)]),
            ..default()
        },
    ));
}

fn build_at_cursor(
    mut commands: Commands,
    ass: Res<AssetServer>,
    cursor: Option<Res<UnderCursor>>,
    action_states: Query<&ActionState<PlayerActions>, With<Player>>,
) {
    for state in action_states.iter() {
        if state.just_pressed(PlayerActions::BuildAtCursor) {
            if let Some(ref cursor) = cursor {
                commands
                    .spawn(SceneBundle {
                        scene: ass.load("dirtpile1.glb#Scene0"),
                        transform: Transform::from_translation(cursor.intersection),
                        ..default()
                    })
                    .insert(Name::new("Dirt Pile"));
            }
        }
    }
}
