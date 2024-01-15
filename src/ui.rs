use std::time::Duration;

use bevy::{
    app::AppExit,
    render::texture::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType},
};
use rand::thread_rng;

use crate::prelude::*;

#[derive(Component)]
pub struct HudScore;

pub fn setup_ui_header(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        font_size: 16.,
        color: UI_TEXT_COLOR,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(UI_HEADER_PX),
                position_type: PositionType::Absolute,
                left: Val::Percent(0.),
                top: Val::Percent(0.),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: UI_COLOR_1.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Percent(1.),
                        ..default()
                    },
                    ..default()
                },
                text: Text::from_section("TaquinPlus", text_style.clone()),
                ..default()
            });
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: UiRect {
                            right: Val::Percent(1.),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text::from_section("", text_style),
                    ..default()
                })
                .insert(HudScore);
        });
}

pub fn update_ui_header(
    mut hud_scores: Query<&mut Text, With<HudScore>>,
    puzzle: Query<&Puzzle, Changed<Puzzle>>,
) {
    if let Ok(puzzle) = puzzle.get_single() {
        let mut hud_score = hud_scores.single_mut();
        hud_score.sections[0].value = format!("Actions: {}", puzzle.actions_count);
    }
}

#[derive(Component)]
pub struct Menu {
    buttons: Vec<(MenuEntry, Entity)>,
    active: usize,
}
impl Menu {
    pub fn set_active(&mut self, menu_entry: &MenuEntry) {
        if let Some(i) = self
            .buttons
            .iter()
            .position(|(entry, _)| entry == menu_entry)
        {
            self.active = i;
        }
    }
    pub fn set_next_active(&mut self) {
        self.active = (self.active + 1) % self.buttons.len();
    }
    pub fn set_prev_active(&mut self) {
        self.active = (self.active + self.buttons.len() - 1) % self.buttons.len();
    }
    pub fn get_active_entry(&self) -> MenuEntry {
        self.buttons[self.active].0
    }
}
#[derive(Component, Clone, Copy, PartialEq, Eq, Event)]
pub enum MenuEntry {
    Continue,
    NewPuzzle,
    Exit,
}
impl MenuEntry {
    pub fn button_text(&self) -> String {
        use MenuEntry::*;
        match self {
            Continue => "Continue",
            NewPuzzle => "New Puzzle",
            Exit => "Exit",
        }
        .to_string()
    }
}

pub fn setup_menu(mut commands: Commands, puzzle: Query<&Puzzle>) {
    let mut button_entries = vec![];
    if puzzle.get_single().is_ok_and(|puzzle| !puzzle.is_solved) {
        button_entries.push(MenuEntry::Continue);
    }
    button_entries.push(MenuEntry::NewPuzzle);
    #[cfg(not(target_family = "wasm"))]
    button_entries.push(MenuEntry::Exit);
    let buttons = button_entries
        .into_iter()
        .map(|button_entry| {
            let entity = commands
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(33.),
                        height: Val::Px(40.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(4.)),
                        margin: UiRect::all(Val::Px(12.)),
                        ..default()
                    },
                    background_color: UI_COLOR_3.into(),
                    border_color: UI_COLOR_1.into(),
                    ..default()
                })
                .insert(button_entry)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        button_entry.button_text(),
                        TextStyle {
                            font_size: 32.0,
                            color: UI_TEXT_COLOR,
                            ..default()
                        },
                    ));
                })
                .id();
            (button_entry, entity)
        })
        .collect::<Vec<_>>();
    commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                top: Val::Px(UI_HEADER_PX),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                //margin: UiRect::all(Val::Px(100.)),
                ..default()
            },
            ..default()
        })
        .push_children(
            &buttons
                .iter()
                .map(|(_, entity)| *entity)
                .collect::<Vec<_>>(),
        )
        .insert(Menu { buttons, active: 0 });
}

pub fn despawn_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn menu_active_update(
    menu: Query<&Menu, Changed<Menu>>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    if let Ok(menu) = menu.get_single() {
        for (i, (_, entity)) in menu.buttons.iter().enumerate() {
            let mut color = backgrounds
                .get_mut(*entity)
                .expect("Inconsistency in menu buttons Entity and spawned entities");
            *color = if i == menu.active {
                UI_COLOR_2.into()
            } else {
                UI_COLOR_3.into()
            };
        }
    }
}
pub fn menu_interaction(
    mut menu: Query<&mut Menu>,
    button_interaction: Query<(&Interaction, &MenuEntry)>,
    input: Res<Input<KeyCode>>,
    mut menu_events: EventWriter<MenuEntry>,
) {
    for (interaction, menu_entry) in button_interaction.iter() {
        let mut menu = menu.single_mut();
        match *interaction {
            Interaction::Pressed => {
                menu_events.send(*menu_entry);
            }
            Interaction::Hovered => {
                menu.set_active(menu_entry);
            }
            Interaction::None => {}
        }
    }
    if input.just_pressed(KeyCode::Down) {
        menu.single_mut().set_next_active();
    }
    if input.just_pressed(KeyCode::Up) {
        menu.single_mut().set_prev_active();
    }
    if input.just_pressed(KeyCode::Return) {
        menu_events.send(menu.single().get_active_entry());
    }
}

pub fn menu_event_handler(
    mut commands: Commands,
    mut menu_events: EventReader<MenuEntry>,
    mut app_exit_events: EventWriter<AppExit>,
    mut next_gamestate: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    puzzle: Query<(Entity, &Puzzle)>,
) {
    for menu_entry in menu_events.read() {
        match menu_entry {
            MenuEntry::Continue => {
                next_gamestate.set(GameState::PuzzleSolve);
            }
            MenuEntry::NewPuzzle => {
                let new_size = if let Ok((entity, puzzle)) = puzzle.get_single() {
                    let cur_size = puzzle.size();
                    commands.entity(entity).despawn_recursive();
                    if cur_size == (7, 7) {
                        (3, 3)
                    } else {
                        (cur_size.0 + 1, cur_size.1 + 1)
                    }
                } else {
                    (3, 3)
                };
                println!("WEB request");
                let image = if let Ok(bytes) = attohttpc::get("https://picsum.photos/1024.webp")
                    .send()
                    .and_then(|resp| resp.bytes())
                {
                    images.add(
                        Image::from_buffer(
                            &bytes,
                            ImageType::Format(ImageFormat::WebP),
                            CompressedImageFormats::NONE,
                            true,
                            ImageSampler::Default.clone(),
                        )
                        .expect("Image could not be loaded"),
                    )
                } else {
                    asset_server.load("images/1.png")
                };
                let mut puzzle = Puzzle::new(image, new_size.0, new_size.1);
                let rng = thread_rng();
                let (n_moves, flip_pct, rot_pct) = match new_size.0 {
                    3 => (5, 0., 0.),
                    4 => (20, 0., 0.),
                    5 => (100, 0.0, 0.2),
                    _ => (1000, 1., 1.),
                };
                puzzle.shuffle(n_moves, flip_pct, rot_pct, rng);
                // Spawn a simple Entity with just a Puzzle
                // All addition entities will be added in a dedicated system
                commands.spawn(puzzle);
                next_gamestate.set(GameState::PuzzleSolve);
            }
            MenuEntry::Exit => {
                app_exit_events.send(AppExit);
            }
        }
    }
}
pub fn puzzle_deblur(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, With<GaussianBlurSettings>)>,
) {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(BLUR_ANIMATION_DURATION),
        GaussianBlurLens {
            start: BLUR,
            end: NO_BLUR,
        },
    );
    let camera_entity = camera.single();
    commands.entity(camera_entity).insert(Animator::new(tween));
}
pub fn puzzle_blur(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, With<GaussianBlurSettings>)>,
) {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(BLUR_ANIMATION_DURATION),
        GaussianBlurLens {
            start: NO_BLUR,
            end: BLUR,
        },
    );
    let camera_entity = camera.single();
    commands.entity(camera_entity).insert(Animator::new(tween));
}
