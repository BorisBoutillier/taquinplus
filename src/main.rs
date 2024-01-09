use bevy::input::common_conditions::input_toggle_active;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::{app::AppExit, asset::AssetMetaCheck};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::TweeningPlugin;
use rand::thread_rng;

mod prelude;
mod puzzle;
mod tile;
use crate::prelude::*;
const HUD_PCT: f32 = 5.0;
//const HUD_BACKGROUND_COLOR: Color = Color::rgb(49. / 255., 44. / 255., 77. / 255.);
const HUD_BACKGROUND_COLOR: Color = Color::rgb(103. / 255., 91. / 255., 153. / 255.);
const MAIN_BACKGROUND_COLOR: Color = Color::rgb(39. / 255., 34. / 255., 61. / 255.);
fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(MAIN_BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TaquinPlus".to_string(),
                resolution: [800.0, 600.0].into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F12)),
        )
        .add_plugins(TweeningPlugin)
        .add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, test_inputs)
        .add_systems(Update, new_puzzle)
        .add_event::<PuzzleAction>()
        .add_systems(Update, handle_puzzle_action_events)
        .add_systems(Update, puzzle_resize)
        .add_systems(Update, asset_animator_system::<Mesh>)
        .add_systems(Update, tile_animation)
        .add_systems(Update, update_hud_score)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = thread_rng();
    let projection = OrthographicProjection {
        far: 1000.,
        near: -1000.,
        ..default()
    };
    commands.spawn(Camera3dBundle {
        projection: Projection::Orthographic(projection),
        transform: Transform::from_xyz(0.0, 0., 20.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    let mut puzzle = Puzzle::new(asset_server.load("images/1.png"), 3, 3);
    puzzle.shuffle(5, 0., 0., &mut rng);
    commands.add(puzzle);

    commands.insert_resource(AmbientLight {
        brightness: 3.0,
        ..default()
    });
}

#[derive(Component)]
struct HudScore;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        font_size: 16.,
        color: Color::ANTIQUE_WHITE,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(HUD_PCT),
                position_type: PositionType::Absolute,
                left: Val::Percent(0.),
                top: Val::Percent(0.),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: HUD_BACKGROUND_COLOR.into(),
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
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(5.),
            height: Val::Percent(5.),
            position_type: PositionType::Absolute,
            left: Val::Percent(95.),
            top: Val::Percent(95.),
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    });
}

fn test_inputs(
    mut puzzle_solution: Query<&mut Visibility, (With<PuzzleSolution>, Without<PuzzleTiles>)>,
    mut puzzle_tiles: Query<&mut Visibility, With<PuzzleTiles>>,
    puzzle_assets: Res<PuzzleAssets>,
    mut puzzle: Query<&mut Puzzle>,
    mut outlines: Query<&mut OutlineVolume>,
    input: Res<Input<KeyCode>>,
    mut puzzle_move_events: EventWriter<PuzzleAction>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::D) {
        puzzle_move_events.send(PuzzleAction::ActiveFlipX);
    }
    if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::S) {
        puzzle_move_events.send(PuzzleAction::ActiveFlipY);
    }
    if input.just_pressed(KeyCode::Q) {
        puzzle_move_events.send(PuzzleAction::ActiveRotateCCW);
    }
    if input.just_pressed(KeyCode::E) {
        puzzle_move_events.send(PuzzleAction::ActiveRotateCW);
    }
    if input.just_pressed(KeyCode::Right) {
        if input.pressed(KeyCode::ShiftLeft) {
            puzzle_move_events.send(PuzzleAction::MoveActiveRight);
        } else {
            puzzle_move_events.send(PuzzleAction::MoveRight);
        }
    }
    if input.just_pressed(KeyCode::Left) {
        if input.pressed(KeyCode::ShiftLeft) {
            puzzle_move_events.send(PuzzleAction::MoveActiveLeft);
        } else {
            puzzle_move_events.send(PuzzleAction::MoveLeft);
        }
    }
    if input.just_pressed(KeyCode::Up) {
        if input.pressed(KeyCode::ShiftLeft) {
            puzzle_move_events.send(PuzzleAction::MoveActiveUp);
        } else {
            puzzle_move_events.send(PuzzleAction::MoveUp);
        }
    }
    if input.just_pressed(KeyCode::Down) {
        if input.pressed(KeyCode::ShiftLeft) {
            puzzle_move_events.send(PuzzleAction::MoveActiveDown);
        } else {
            puzzle_move_events.send(PuzzleAction::MoveDown);
        }
    }
    #[cfg(not(target_family = "wasm"))]
    if input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }

    // Handle display of the solution overlay pressing/releasing a key
    // Beware that some kind of puzzle don't have a solution that can be shown
    if input.just_pressed(KeyCode::ControlLeft) {
        let puzzle = puzzle.single();
        if !puzzle.is_solved {
            for mut solution in puzzle_solution.iter_mut() {
                *solution = Visibility::Visible;
            }
            for mut tiles in puzzle_tiles.iter_mut() {
                *tiles = Visibility::Hidden;
            }
        }
    }
    if input.just_released(KeyCode::ControlLeft) {
        for mut solution in puzzle_solution.iter_mut() {
            *solution = Visibility::Hidden;
        }
        for mut tiles in puzzle_tiles.iter_mut() {
            *tiles = Visibility::Visible;
        }
    }
    if input.just_pressed(KeyCode::Space) {
        let mut puzzle = puzzle.single_mut();
        puzzle.show_errors = true;
        puzzle.show_outlines(&mut outlines, puzzle_assets.as_ref());
    }
    if input.just_released(KeyCode::Space) {
        let mut puzzle = puzzle.single_mut();
        puzzle.show_errors = false;
        puzzle.show_outlines(&mut outlines, puzzle_assets.as_ref());
    }
}
fn new_puzzle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    puzzle: Query<(Entity, &Puzzle)>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::N) {
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
        let mut puzzle = Puzzle::new(asset_server.load("images/1.png"), new_size.0, new_size.1);
        let rng = thread_rng();
        let (n_moves, flip_pct, rot_pct) = match new_size.0 {
            3 => (5, 0., 0.),
            4 => (20, 0., 0.),
            5 => (100, 0.0, 0.2),
            _ => (1000, 1., 1.),
        };
        puzzle.shuffle(n_moves, flip_pct, rot_pct, rng);
        commands.add(puzzle);
    }
}

fn puzzle_resize(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut puzzle_transform: Query<&mut Transform, With<Puzzle>>,
    added_puzzle: Query<(), Added<Puzzle>>,
    resize_events: EventReader<WindowResized>,
) {
    if !resize_events.is_empty() || !added_puzzle.is_empty() {
        let primary_window = primary_window.single();
        let mut puzzle_transform = puzzle_transform.single_mut();
        let height = primary_window.resolution.physical_height() as f32;
        let width = primary_window.resolution.physical_width() as f32;
        let min = (1. - HUD_PCT / 100.) * 0.95 * height.min(width);
        puzzle_transform.scale = Vec3::new(min, min, 1.);
        puzzle_transform.translation.y = -(height / 2.) * HUD_PCT / 100.;
    }
}

fn update_hud_score(
    mut hud_scores: Query<&mut Text, With<HudScore>>,
    puzzle: Query<&Puzzle, Changed<Puzzle>>,
) {
    for puzzle in puzzle.iter() {
        let mut hud_score = hud_scores.single_mut();
        hud_score.sections[0].value = format!("Actions: {}", puzzle.actions_count);
    }
}
