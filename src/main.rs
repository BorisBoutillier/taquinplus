use bevy::asset::AssetMetaCheck;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod prelude;
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
        .run();
}

#[derive(Resource)]
pub struct Puzzle {
    pub current_tile: Option<Entity>,
    pub hole: (usize, usize),
    pub tiles: Vec<Vec<Option<Entity>>>,
    pub len_x: usize,
    pub len_y: usize,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let image_handle = asset_server.load("images/1.png");
    let len_x = 5;
    let len_y = 5;
    let hole = (4, 0);
    let tiles: Vec<Vec<Option<Entity>>> = (0..len_x)
        .map(|x| {
            (0..len_y)
                .map(|y| {
                    if (x, y) == hole {
                        None
                    } else {
                        Some(
                            commands
                                .spawn(PbrBundle {
                                    material: materials.add(StandardMaterial {
                                        base_color_texture: Some(image_handle.clone()),
                                        reflectance: 0.0,
                                        ..default()
                                    }),
                                    transform: Transform::from_scale(Vec3::new(93.0, 93.0, 5.)),
                                    ..default()
                                })
                                .insert(Piece {
                                    x,
                                    y,
                                    len_x: 5,
                                    len_y: 5,
                                })
                                .insert(Position { x, y, len_x, len_y })
                                .insert(Flipped {
                                    flipped_x: false,
                                    flipped_y: false,
                                })
                                .insert(Rotated::default())
                                .id(),
                        )
                    }
                })
                .collect()
        })
        .collect();
    commands.insert_resource(Puzzle {
        tiles,
        hole,
        current_tile: None,
        len_x,
        len_y,
    });
    commands.insert_resource(AmbientLight {
        brightness: 3.0,
        ..default()
    });
}

fn test_inputs(
    mut puzzle: ResMut<Puzzle>,
    input: Res<Input<KeyCode>>,
    mut flips: Query<&mut Flipped>,
    mut rotations: Query<&mut Rotated>,
    mut positions: Query<&mut Position>,
) {
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
    let (hole_x, hole_y) = puzzle.hole;
    let mut new_hole_x = hole_x;
    let mut new_hole_y = hole_y;
    if input.just_pressed(KeyCode::Right) && new_hole_x > 0 {
        new_hole_x -= 1;
    }
    if input.just_pressed(KeyCode::Left) && new_hole_x < puzzle.len_x - 1 {
        new_hole_x += 1;
    }
    if input.just_pressed(KeyCode::Up) && new_hole_y > 0 {
        new_hole_y -= 1;
    }
    if input.just_pressed(KeyCode::Down) && new_hole_y < puzzle.len_y - 1 {
        new_hole_y += 1;
    }
    if new_hole_x != hole_x || new_hole_y != hole_y {
        let tile = puzzle.tiles[new_hole_x][new_hole_y].take().expect("Oops");
        let mut tile_position = positions.get_mut(tile).expect("Oops");
        tile_position.x = hole_x;
        tile_position.y = hole_y;
        puzzle.tiles[hole_x][hole_y] = Some(tile);
        puzzle.hole.0 = new_hole_x;
        puzzle.hole.1 = new_hole_y;
        puzzle.current_tile = Some(tile);
    }
}
