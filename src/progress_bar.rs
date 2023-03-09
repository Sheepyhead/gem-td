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
                    position: UiRect::new(
                        Val::Px(position.x - 16.0),
                        Val::Auto,
                        Val::Px(position.y - 2.5),
                        Val::Auto,
                    ),
                    size: Size::new(Val::Px(32.0), Val::Px(5.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        background_color: foreground_color.into(),
                        style: Style {
                            size: Size::new(Val::Percent(progress * 100.0), Val::Percent(100.0)),
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
