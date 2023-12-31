use bevy::window::WindowResized;
use bevy::{app::AppExit, asset::AssetMetaCheck};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod prelude;
mod puzzle;
mod tile;
use crate::prelude::*;
fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TaquinPlus".to_string(),
                resolution: [800.0, 600.0].into(),
                ..default()
            }),
            ..default()
        }))
        //.add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, update_tile_flip)
        .add_systems(Update, update_tile_rotation)
        .add_systems(Update, update_tile_position)
        .add_systems(Update, test_inputs)
        .add_event::<PuzzleMoveEvent>()
        .add_systems(Update, handle_action_events)
        .add_systems(Update, update_puzzle_on_resize)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
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
    let puzzle = Puzzle::new(asset_server.load("images/1.png"), 5, 5);
    puzzle.spawn(&mut commands, materials);

    commands.insert_resource(AmbientLight {
        brightness: 3.0,
        ..default()
    });
}

fn test_inputs(
    mut puzzle: Query<(&Puzzle, &mut Visibility)>,
    input: Res<Input<KeyCode>>,
    mut flips: Query<&mut Flipped>,
    mut rotations: Query<&mut Rotated>,
    mut puzzle_move_events: EventWriter<PuzzleMoveEvent>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    let (puzzle, mut puzzle_visibility) = puzzle.single_mut();
    if input.just_pressed(KeyCode::Space) {
        *puzzle_visibility = match *puzzle_visibility {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            Visibility::Inherited => panic!(),
        };
    }
    if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::D) {
        if let Some(tile) = puzzle.current_tile {
            let mut flip = flips.get_mut(tile).expect("Oops");
            flip.flipped_x = !flip.flipped_x;
        }
    }
    if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::D) {
        if let Some(tile) = puzzle.current_tile {
            let mut flip = flips.get_mut(tile).expect("Oops");
            flip.flipped_x = !flip.flipped_x;
        }
    }
    if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::S) {
        if let Some(tile) = puzzle.current_tile {
            let mut flip = flips.get_mut(tile).expect("Oops");
            flip.flipped_y = !flip.flipped_y;
        }
    }
    if input.just_pressed(KeyCode::Q) {
        if let Some(tile) = puzzle.current_tile {
            let mut rotation = rotations.get_mut(tile).expect("Oops");
            rotation.rot_ccw();
        }
    }
    if input.just_pressed(KeyCode::E) {
        if let Some(tile) = puzzle.current_tile {
            let mut rotation = rotations.get_mut(tile).expect("Oops");
            rotation.rot_cw();
        }
    }
    if input.just_pressed(KeyCode::Right) {
        puzzle_move_events.send(PuzzleMoveEvent::MoveRight);
    }
    if input.just_pressed(KeyCode::Left) {
        puzzle_move_events.send(PuzzleMoveEvent::MoveLeft);
    }
    if input.just_pressed(KeyCode::Up) {
        puzzle_move_events.send(PuzzleMoveEvent::MoveUp);
    }
    if input.just_pressed(KeyCode::Down) {
        puzzle_move_events.send(PuzzleMoveEvent::MoveDown);
    }
    if input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

fn update_puzzle_on_resize(
    windows: Query<&Window>,
    mut puzzle_tf: Query<&mut Transform, With<Puzzle>>,
    mut resize_events: EventReader<WindowResized>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut puzzle_tf = puzzle_tf.single_mut();
        let min = 0.95
            * window
                .resolution
                .physical_height()
                .min(window.resolution.physical_width()) as f32;
        puzzle_tf.scale = Vec3::new(min, min, 1.);
    }
}
