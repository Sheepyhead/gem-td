use bevy::prelude::*;

#[derive(Component)]
pub struct ProgressBar {
    pub target: Entity,
}

impl ProgressBar {
    pub fn spawn(
        position: Vec2,
        foreground_color: Color,
        background_color: Color,
        progress: f32,
        target: Entity,
        commands: &mut Commands,
    ) -> Entity {
        commands
            .spawn(NodeBundle {
                background_color: background_color.into(),
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(position.x - 16.0),
                    right: Val::Auto,
                    top: Val::Px(position.y - 2.5),
                    bottom: Val::Auto,
                    width: Val::Px(32.0),
                    height: Val::Px(5.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        background_color: foreground_color.into(),
                        style: Style {
                            width: Val::Percent(progress * 100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    },
                    ProgressBar { target },
                ));
            })
            .id()
    }
}
