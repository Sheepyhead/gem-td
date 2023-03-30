use bevy::prelude::*;

pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems((Sidebar::spawn,));
    }
}

#[derive(Component)]
pub struct Sidebar;

impl Sidebar {
    pub fn spawn(mut commands: Commands, ass: Res<AssetServer>) {
        let full_screen = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::ColumnReverse,
                        size: Size::all(Val::Percent(100.)),
                        ..default()
                    },
                    ..default()
                },
                Sidebar,
            ))
            .id();

        let sidebar_background = commands
            .spawn(NodeBundle {
                background_color: Color::DARK_GRAY.into(),
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect {
                        top: Val::Percent(1.),
                        bottom: Val::Percent(1.),
                        left: Val::Percent(5.),
                        right: Val::Percent(5.),
                    },
                    size: Size::height(Val::Percent(100.)),
                    align_self: AlignSelf::End,
                    ..default()
                },
                ..default()
            })
            .id();

        let title = commands
            .spawn(TextBundle {
                text: Text::from_section(
                    "Title",
                    TextStyle {
                        font: ass.load("Pixelcastle-Regular.otf"),
                        font_size: 25.,
                        color: Color::ANTIQUE_WHITE,
                    },
                ),
                ..default()
            })
            .id();

        let button = commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::all(Val::Px(50.)),
                    ..default()
                },
                background_color: Color::PINK.into(),
                ..default()
            })
            .id();
        let icon = commands
            .spawn(ImageBundle {
                image: UiImage::new(ass.load("chipped.PNG")),
                style: Style {
                    size: Size::width(Val::Percent(100.)),
                    ..default()
                },
                ..default()
            })
            .id();

        commands.entity(full_screen).add_child(sidebar_background);
        commands
            .entity(sidebar_background)
            .add_child(title)
            .add_child(button);
        commands.entity(button).add_child(icon);
    }

    fn _despawn(mut commands: Commands, sidebar: Query<Entity, With<Sidebar>>) {
        for sidebar in &sidebar {
            commands.entity(sidebar).despawn_recursive();
        }
    }
}
