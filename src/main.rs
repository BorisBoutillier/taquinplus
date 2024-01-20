use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::TweeningPlugin;

mod game_state;
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
                resizable: true,
                fit_canvas_to_parent: true,
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
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>()
                .disable::<DefaultHighlightingPlugin>(),
        )
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
            puzzle_solving_interaction.run_if(in_state(GameState::PuzzleSolving)),
        )
        .add_systems(OnEnter(GameState::PuzzleSolving), spawn_puzzle_entities)
        .add_systems(OnEnter(GameState::Menu), puzzle_blur)
        .add_systems(OnExit(GameState::Menu), puzzle_deblur)
        .add_systems(OnEnter(GameState::PuzzleSolved), show_full_puzzle)
        .add_systems(
            Update,
            puzzle_solved_interaction.run_if(in_state(GameState::PuzzleSolved)),
        )
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

fn puzzle_resize(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut puzzle_transform: Query<&mut Transform, With<Puzzle>>,
    added_puzzle: Query<(), (With<Puzzle>, Added<GlobalTransform>)>,
    mut resize_events: EventReader<WindowResized>,
) {
    if !resize_events.is_empty() || !added_puzzle.is_empty() {
        resize_events.clear();
        if let Ok(mut puzzle_transform) = puzzle_transform.get_single_mut() {
            let primary_window = primary_window.single();
            let height = primary_window.height() - UI_HEADER_PX;
            let width = primary_window.width();
            let min = 0.95 * height.min(width);
            puzzle_transform.scale = Vec3::new(min, min, 1.);
            puzzle_transform.translation.y = -UI_HEADER_PX / 2.;
        }
    }
}

fn show_fps(input: Res<Input<KeyCode>>, diag: Res<DiagnosticsStore>) {
    if input.just_pressed(KeyCode::F11) {
        if let Some(fps) = diag
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            info!("FPS: {:.1}", fps);
        }
    }
}
