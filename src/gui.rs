use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    controls::SelectedTower,
    towers::{
        Cooldown, LaserAttack, PickSelectedTower, RandomLevel, RefineAndPickSelectedTower,
        RemoveSelectedTower,
    },
    Phase,
};

pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems((SidebarFullscreen::spawn,))
            .add_systems((
                event_buttons::<PickSelectedTower>.in_set(OnUpdate(Phase::Pick)),
                event_buttons::<RefineAndPickSelectedTower>.in_set(OnUpdate(Phase::Pick)),
                event_buttons::<RemoveSelectedTower>.in_set(OnUpdate(Phase::Pick)),
                event_buttons::<RemoveSelectedTower>.in_set(OnUpdate(Phase::Build)),
                UpgradeChanceButton::interaction,
                UpgradeChanceButton::update,
                SelectedText::on_update,
                show_pickable_button,
                show_refine_and_pick_button,
                show_remove_button,
            ));
    }
}

#[derive(Component)]
pub struct SidebarFullscreen;

#[derive(Component)]
pub struct Sidebar;

impl SidebarFullscreen {
    pub fn spawn(mut commands: Commands, ass: Res<AssetServer>, upgrade_chance: Res<RandomLevel>) {
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
                SidebarFullscreen,
            ))
            .id();

        let sidebar_background = commands
            .spawn((
                NodeBundle {
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
                },
                Sidebar,
            ))
            .id();

        let title = commands
            .spawn(TextBundle {
                text: Text::from_section(
                    "Title",
                    TextStyle {
                        font: ass.load("Mukta-Regular.ttf"),
                        font_size: 45.,
                        color: Color::ANTIQUE_WHITE,
                    },
                ),
                style: Style {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ..default()
            })
            .id();

        let selected_text = commands
            .spawn((
                TextBundle {
                    text: Text::default(),
                    style: Style {
                        align_self: AlignSelf::Start,
                        ..default()
                    },
                    ..default()
                },
                SelectedText,
            ))
            .id();

        let button_bar = commands
            .spawn((NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },))
            .id();

        let upgrade_chance_button_text = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(
                        format!("{}", **upgrade_chance),
                        TextStyle {
                            font: ass.load("Mukta-Regular.ttf"),
                            font_size: 40.,
                            color: Color::BLACK,
                        },
                    ),
                    ..default()
                },
                UpgradeChanceButtonText,
            ))
            .id();

        let buttons = [
            commands
                .spawn((EventButtonBundle {
                    button: ButtonBundle {
                        style: Style {
                            size: Size::all(Val::Px(50.)),
                            ..default()
                        },
                        background_color: Color::PINK.into(),
                        ..default()
                    },
                    event: EventButton::<PickSelectedTower>::new(),
                },))
                .id(),
            commands
                .spawn((EventButtonBundle {
                    button: ButtonBundle {
                        style: Style {
                            size: Size::all(Val::Px(50.)),
                            ..default()
                        },
                        background_color: Color::GREEN.into(),
                        ..default()
                    },
                    event: EventButton::<RefineAndPickSelectedTower>::new(),
                },))
                .id(),
            commands
                .spawn((EventButtonBundle {
                    button: ButtonBundle {
                        style: Style {
                            size: Size::all(Val::Px(50.)),
                            ..default()
                        },
                        background_color: Color::TEAL.into(),
                        ..default()
                    },
                    event: EventButton::<RemoveSelectedTower>::new(),
                },))
                .id(),
            commands
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::all(Val::Px(50.)),
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::YELLOW.into(),
                        ..default()
                    },
                    UpgradeChanceButton,
                ))
                .add_child(upgrade_chance_button_text)
                .id(),
        ];

        commands.entity(full_screen).add_child(sidebar_background);

        commands
            .entity(sidebar_background)
            .add_child(title)
            .add_child(selected_text)
            .add_child(button_bar);

        commands.entity(button_bar).push_children(&buttons);
    }

    fn _despawn(mut commands: Commands, sidebar: Query<Entity, With<SidebarFullscreen>>) {
        for sidebar in &sidebar {
            commands.entity(sidebar).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct UpgradeChanceButton;

#[derive(Component)]
struct UpgradeChanceButtonText;

impl UpgradeChanceButton {
    fn interaction(
        mut random_level: ResMut<RandomLevel>,
        buttons: Query<&Interaction, (With<UpgradeChanceButton>, Changed<Interaction>)>,
    ) {
        for interaction in &buttons {
            if let Interaction::Clicked = interaction {
                **random_level += 1;
            }
        }
    }

    fn update(
        ass: Res<AssetServer>,
        random_level: Res<RandomLevel>,
        mut text: Query<&mut Text, With<UpgradeChanceButtonText>>,
    ) {
        if random_level.is_changed() {
            for mut text in &mut text {
                *text = Text::from_section(
                    format!("{}", **random_level),
                    TextStyle {
                        font: ass.load("Mukta-Regular.ttf"),
                        font_size: 40.,
                        color: Color::BLACK,
                    },
                );
            }
        }
    }
}

#[derive(Component)]
struct SelectedText;

impl SelectedText {
    fn on_update(
        ass: Res<AssetServer>,
        selected: Option<Res<SelectedTower>>,
        mut text: Query<&mut Text, With<SelectedText>>,
        tower_stats: Query<(&Name, Option<&LaserAttack>, Option<&Cooldown>)>,
    ) {
        if let Some(selected) = selected {
            if selected.is_changed() {
                if let Ok((name, attack, cooldown)) = tower_stats.get(selected.tower) {
                    let mut text = text.single_mut();
                    let mut style = TextStyle {
                        font: ass.load("Mukta-Regular.ttf"),
                        font_size: 30.,
                        color: if let Some(LaserAttack { color, .. }) = attack {
                            *color
                        } else {
                            Color::ANTIQUE_WHITE
                        },
                    };
                    let mut text_section =
                        Text::from_sections([TextSection::new(format!("{name}\n"), style.clone())]);
                    style.color = Color::ANTIQUE_WHITE;
                    if let Some(LaserAttack {
                        range,
                        damage,
                        hits,
                        ..
                    }) = attack
                    {
                        text_section
                            .sections
                            .push(TextSection::new(format!("Range: {range}\n"), style.clone()));
                        text_section.sections.push(TextSection::new(
                            format!("Damage: {damage}\n"),
                            style.clone(),
                        ));
                        text_section.sections.push(TextSection::new(
                            format!("Targets: {hits}\n"),
                            style.clone(),
                        ));
                    }
                    if let Some(Cooldown(timer)) = cooldown {
                        text_section.sections.push(TextSection::new(
                            format!("Attack speed: {}", timer.duration().as_secs_f32()),
                            style,
                        ));
                    }
                    *text = text_section;
                }
            }
        } else {
            let mut text = text.single_mut();
            *text = Text::default();
        }
    }
}

#[derive(Bundle)]
struct EventButtonBundle<T: Default + Send + Sync + 'static> {
    #[bundle]
    button: ButtonBundle,
    event: EventButton<T>,
}

#[derive(Component)]
struct EventButton<T: Default>(PhantomData<T>);

impl<T: Default> EventButton<T> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

fn event_buttons<T: Default + Send + Sync + 'static>(
    mut events: EventWriter<T>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<EventButton<T>>)>,
) {
    for interaction in &buttons {
        if let Interaction::Clicked = interaction {
            events.send(T::default());
        }
    }
}

fn show_pickable_button(
    selected: Option<Res<SelectedTower>>,
    mut buttons: Query<(&mut Style, &mut Visibility), With<EventButton<PickSelectedTower>>>,
) {
    if let Some(selected) = selected {
        if selected.is_changed() {
            for (mut style, mut visibility) in &mut buttons {
                (style.display, *visibility) = if selected.pickable {
                    (Display::Flex, Visibility::Inherited)
                } else {
                    (Display::None, Visibility::Hidden)
                }
            }
        }
    } else {
        for (mut style, mut visibility) in &mut buttons {
            style.display = Display::None;
            *visibility = Visibility::Hidden;
        }
    }
}

fn show_refine_and_pick_button(
    selected: Option<Res<SelectedTower>>,
    mut buttons: Query<
        (&mut Style, &mut Visibility),
        With<EventButton<RefineAndPickSelectedTower>>,
    >,
) {
    if let Some(selected) = selected {
        if selected.is_changed() {
            for (mut style, mut visibility) in &mut buttons {
                (style.display, *visibility) = if selected.refinable {
                    (Display::Flex, Visibility::Inherited)
                } else {
                    (Display::None, Visibility::Hidden)
                }
            }
        }
    } else {
        for (mut style, mut visibility) in &mut buttons {
            style.display = Display::None;
            *visibility = Visibility::Hidden;
        }
    }
}

fn show_remove_button(
    selected: Option<Res<SelectedTower>>,
    phase: Res<State<Phase>>,
    mut buttons: Query<(&mut Style, &mut Visibility), With<EventButton<RemoveSelectedTower>>>,
) {
    match phase.0 {
        Phase::Build | Phase::Pick => {
            if let Some(selected) = selected {
                if selected.is_changed() {
                    for (mut style, mut visibility) in &mut buttons {
                        (style.display, *visibility) = if selected.removable {
                            (Display::Flex, Visibility::Inherited)
                        } else {
                            (Display::None, Visibility::Hidden)
                        }
                    }
                }
            } else {
                for (mut style, mut visibility) in &mut buttons {
                    style.display = Display::None;
                    *visibility = Visibility::Hidden;
                }
            }
        }
        Phase::Spawn => {
            for (mut style, mut visibility) in &mut buttons {
                style.display = Display::None;
                *visibility = Visibility::Hidden;
            }
        }
    }
}
