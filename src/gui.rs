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
        let sidebar = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    size: Size::all(Val::Percent(100.)),
                    ..default()
                },
                ..default()
            })
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
                        left: Val::Percent(5.),
                        right: Val::Percent(5.),
                        ..default()
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

        commands.entity(sidebar).add_child(sidebar_background);
        commands.entity(sidebar_background).add_child(title);
    }
}
