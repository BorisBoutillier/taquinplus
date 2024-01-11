use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::TweeningPlugin;

mod game_state;
mod gaussian_blur;
mod prelude;
mod puzzle;
mod tile;
mod ui;
use crate::prelude::*;
fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(MAIN_BACKGROUND_COLOR))
        .add_state::<GameState>()
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
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins(GaussianBlurPlugin)
        .add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin))
        .add_systems(Startup, setup)
        .add_event::<PuzzleAction>()
        .add_event::<MenuEntry>()
        .add_systems(Update, handle_puzzle_action_events)
        .add_systems(Update, puzzle_resize)
        .add_systems(Update, asset_animator_system::<Mesh>)
        .add_systems(Update, component_animator_system::<GaussianBlurSettings>)
        .add_systems(Update, tile_animation)
        .add_systems(Startup, setup_ui_header)
        .add_systems(Update, update_ui_header)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(OnExit(GameState::Menu), despawn_menu)
        .add_systems(
            Update,
            (menu_active_update, menu_interaction, menu_event_handler)
                .run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            Update,
            puzzle_solve_interation.run_if(in_state(GameState::PuzzleSolve)),
        )
        .add_systems(OnEnter(GameState::PuzzleSolve), puzzle_deblur)
        .add_systems(OnExit(GameState::PuzzleSolve), puzzle_blur)
        .add_systems(Update, show_fps)
        .run();
}

fn setup(mut commands: Commands) {
    let projection = OrthographicProjection {
        far: 1000.,
        near: -1000.,
        ..default()
    };
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Orthographic(projection),
            transform: Transform::from_xyz(0.0, 0., 20.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BLUR,
    ));

    commands.insert_resource(AmbientLight {
        brightness: 3.0,
        ..default()
    });
}

fn puzzle_solve_interation(
    mut puzzle_solution: Query<&mut Visibility, (With<PuzzleSolution>, Without<PuzzleTiles>)>,
    mut puzzle_tiles: Query<&mut Visibility, With<PuzzleTiles>>,
    mut puzzle: Query<(&mut Puzzle, &PuzzleAssets)>,
    mut outlines: Query<&mut OutlineVolume>,
    input: Res<Input<KeyCode>>,
    mut puzzle_move_events: EventWriter<PuzzleAction>,
    mut next_gamestate: ResMut<NextState<GameState>>,
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
    // Handle display of the solution overlay pressing/releasing a key
    // Beware that some kind of puzzle don't have a solution that can be shown
    if input.just_pressed(KeyCode::ControlLeft) {
        if let Ok((puzzle, _)) = puzzle.get_single() {
            if !puzzle.is_solved {
                for mut solution in puzzle_solution.iter_mut() {
                    *solution = Visibility::Visible;
                }
                for mut tiles in puzzle_tiles.iter_mut() {
                    *tiles = Visibility::Hidden;
                }
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
        if let Ok((mut puzzle, puzzle_assets)) = puzzle.get_single_mut() {
            puzzle.show_errors = true;
            puzzle.show_outlines(&mut outlines, puzzle_assets);
        }
    }
    if input.just_released(KeyCode::Space) {
        if let Ok((mut puzzle, puzzle_assets)) = puzzle.get_single_mut() {
            puzzle.show_errors = false;
            puzzle.show_outlines(&mut outlines, puzzle_assets);
        }
    }
    if input.just_pressed(KeyCode::Escape) {
        next_gamestate.set(GameState::Menu);
    }
}
fn puzzle_resize(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut puzzle_transform: Query<&mut Transform, With<Puzzle>>,
    added_puzzle: Query<(), Added<Puzzle>>,
    resize_events: EventReader<WindowResized>,
) {
    if !resize_events.is_empty() || !added_puzzle.is_empty() {
        if let Ok(mut puzzle_transform) = puzzle_transform.get_single_mut() {
            let primary_window = primary_window.single();
            let height = primary_window.resolution.physical_height() as f32 - UI_HEADER_PX;
            let width = primary_window.resolution.physical_width() as f32;
            let min = 0.95 * height.min(width);
            puzzle_transform.scale = Vec3::new(min, min, 1.);
            puzzle_transform.translation.y = -UI_HEADER_PX / 2.;
        }
    }
}

fn show_fps(input: Res<Input<KeyCode>>, diag: Res<DiagnosticsStore>) {
    if input.just_pressed(KeyCode::F) {
        if let Some(fps) = diag
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            println!("FPS: {:.1}", fps);
        }
    }
}
