use crate::prelude::*;
use bevy::ecs::system::Command;
use grid::Grid;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;

// Coordinate for tile in the puzzle
// .0 is the row
// .1 is the column
pub type Coord = (usize, usize);

// Tag component for the parent of all solution tiles
#[derive(Component)]
pub struct PuzzleSolution;

// Tag component for the parent of all the current tiles
#[derive(Component)]
pub struct PuzzleTiles;

// Defines is this Tile entity is the Active one
#[derive(Component)]
pub struct Active(bool);

#[derive(Resource)]
pub struct PuzzleAssets {
    tile_scale: Vec3,
    active_tile_scale: Vec3,
}

#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub active: Coord,
    pub hole: Coord,
    pub tiles: Grid<Option<Tile>>,
}
impl Puzzle {
    pub fn new(image: Handle<Image>, width: usize, height: usize) -> Self {
        let hole = (0, width - 1);
        let tiles = Grid::from_vec(
            (0..height)
                .flat_map(|y| {
                    (0..width)
                        .map(|x| {
                            if (y, x) == hole {
                                None
                            } else {
                                Some(Tile::new((y, x)))
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            width,
        );
        Puzzle {
            image,
            active: hole,
            hole,
            tiles,
        }
    }
    pub fn get_active_tile_mut(&mut self) -> &mut Option<Tile> {
        self.tiles
            .get_mut(self.active.0, self.active.1)
            .expect("Invalid Active tile")
    }
    pub fn get_tile_entity(&mut self, position: Coord) -> Option<Entity> {
        self.tiles
            .get(position.0, position.1)
            .and_then(|tile| tile.as_ref())
            .and_then(|tile| tile.entity)
    }
    pub fn size(&self) -> Coord {
        self.tiles.size()
    }
    pub fn shuffle(
        &mut self,
        n_moves: usize,
        flip_pct: f64,
        rotation_pct: f64,
        mut rng: impl RngCore,
    ) {
        let mut reverse_move = None;
        for _ in 0..n_moves {
            let mut possible_moves = self.get_valid_moves();
            possible_moves.retain(|action| Some(*action) != reverse_move);
            let action = possible_moves
                .choose(&mut rng)
                .expect("No possible move found");
            reverse_move = Some(action.reverse());
            self.apply_move_event(*action);
            if let Some(active_tile) = self.get_active_tile_mut() {
                if rng.gen_bool(flip_pct) {
                    let what = rng.gen_range(1..=3u8);
                    if what & 1 == 1 {
                        active_tile.flip_x();
                    }
                    if what & 2 == 2 {
                        active_tile.flip_y();
                    }
                }
                if rng.gen_bool(rotation_pct) {
                    for _ in 0..(rng.gen_range(1..=3u8)) {
                        active_tile.rotate_cw();
                    }
                }
            }
        }
        // After a shuffle we want the active 'tile' to be the hole, not the last moved tiled during shuffling
        self.active = self.hole;
    }
    pub fn is_solved(&self) -> bool {
        let mut incorrect_placement = 0;
        let mut incorrect_flip = 0;
        let mut incorrect_rotation = 0;
        for (coord, tile) in self.tiles.indexed_iter() {
            if let Some(tile) = tile {
                if tile.position != coord {
                    incorrect_placement += 1;
                }
                if tile.is_flipped() {
                    incorrect_flip += 1;
                }
                if tile.is_rotated() {
                    incorrect_rotation += 1;
                }
            }
        }
        incorrect_placement == 0 && incorrect_flip == 0 && incorrect_rotation == 0
    }
    pub fn apply_move_event(&mut self, event: PuzzleAction) -> (Option<Entity>, Coord) {
        use PuzzleAction::*;
        let mut position = self.hole;
        let size = self.size();
        match event {
            MoveLeft => position.1 = (position.1 + 1).min(size.1 - 1),
            MoveRight => position.1 = position.1.max(1) - 1,
            MoveUp => position.0 = position.0.max(1) - 1,
            MoveDown => position.0 = (position.0 + 1).min(size.0 - 1),
            _ => panic!("Not a Move event: {:?}", event),
        }
        if let Some(tile) = self.tiles.get_mut(position.0, position.1).unwrap().take() {
            let destination = self.hole;
            self.active = destination;
            let entity = tile.entity;
            self.tiles[destination] = Some(tile);
            self.hole = position;
            (entity, destination)
        } else {
            (None, position)
        }
    }
    fn get_valid_moves(&self) -> Vec<PuzzleAction> {
        use PuzzleAction::*;
        let size = self.size();
        let mut actions = vec![];
        if self.hole.0 > 0 {
            actions.push(MoveUp);
        }
        if self.hole.0 < size.0 - 1 {
            actions.push(MoveDown);
        }
        if self.hole.1 > 0 {
            actions.push(MoveRight);
        }
        if self.hole.1 < size.1 - 1 {
            actions.push(MoveLeft);
        }
        actions
    }
}
fn tile_translation_from_position(position: (usize, usize), size: (usize, usize)) -> Vec3 {
    Vec3::new(
        (position.1 as isize - (size.1 as isize / 2)) as f32 / size.1 as f32,
        (position.0 as isize - (size.0 as isize / 2)) as f32 / size.0 as f32,
        0.,
    )
}

impl Command for Puzzle {
    fn apply(mut self, world: &mut World) {
        let tile_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("No Resource Assets<StandardMaterial>");
            materials.add(StandardMaterial {
                base_color_texture: Some(self.image.clone()),
                reflectance: 0.0,
                ..default()
            })
        };
        let size = self.size();
        let tile_scale = 0.93 / (size.0.max(size.1) as f32);
        let tile_scale = Vec3::new(tile_scale, tile_scale, 5.);
        let active_tile_scale = tile_scale * 1.05;
        let tile_transform = Transform::from_scale(tile_scale);
        let mut solution_tiles = vec![];
        self.tiles.indexed_iter_mut().for_each(|(index, tile)| {
            if let Some(tile) = tile.as_mut() {
                let mesh = {
                    let mut meshes = world
                        .get_resource_mut::<Assets<Mesh>>()
                        .expect("No Resource Assets<Mesh>");

                    meshes.add(tile.compute_mesh(size))
                };
                tile.entity = Some(
                    world
                        .spawn(PbrBundle {
                            material: tile_material.clone(),
                            mesh: mesh.clone(),
                            transform: tile_transform
                                .with_rotation(tile.compute_rotation())
                                .with_translation(tile_translation_from_position(index, size)),
                            ..default()
                        })
                        .insert(Active(index == self.active))
                        .id(),
                );
                // Duplicate the tile to add to the PuzzleSolution at the real tile position
                // Need a duplicate mesh, because it must not be flipped when the main tile is flipped
                let solution_mesh = {
                    let mut meshes = world
                        .get_resource_mut::<Assets<Mesh>>()
                        .expect("No Resource Assets<Mesh>");

                    meshes.add(compute_tile_mesh(size, tile.position, false, false))
                };
                solution_tiles.push(
                    world
                        .spawn(PbrBundle {
                            material: tile_material.clone(),
                            mesh: solution_mesh,
                            transform: tile_transform.with_translation(
                                tile_translation_from_position(tile.position, size),
                            ),
                            ..default()
                        })
                        .id(),
                );
            }
        });
        // Spawn the puzzle solution parent
        let puzzle_solution = world
            .spawn(SpatialBundle {
                visibility: Visibility::Hidden,
                // Have the solution 'on top' of the puzzle, so positive Z
                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                ..default()
            })
            .push_children(&solution_tiles)
            .insert(Name::new("Solution"))
            .insert(PuzzleSolution)
            .id();
        // Spawn the puzzle tiles parent
        // Spawn the main puzzle
        let puzzle_tiles = world
            .spawn(SpatialBundle::default())
            .push_children(
                &self
                    .tiles
                    .iter()
                    .filter_map(|tile| tile.as_ref().and_then(|tile| tile.entity))
                    .collect::<Vec<_>>(),
            )
            .insert(Name::new("Tiles"))
            .insert(PuzzleTiles)
            .id();
        // Spawn the puzzle entity, with solution and tiles children
        world
            .spawn(SpatialBundle::default())
            .insert(Name::new("Puzzle"))
            .insert(self)
            .add_child(puzzle_solution)
            .add_child(puzzle_tiles);
        // Create the resource containing all the needed asset handles for the Puzzle
        world.insert_resource(PuzzleAssets {
            tile_scale,
            active_tile_scale,
        });
    }
}

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    #[allow(dead_code)]
    ActiveFlipX,
    ActiveFlipY,
    ActiveRotateCW,
    ActiveRotateCCW,
}
impl PuzzleAction {
    pub fn reverse(&self) -> PuzzleAction {
        use PuzzleAction::*;
        match self {
            MoveLeft => MoveRight,
            MoveRight => MoveLeft,
            MoveUp => MoveDown,
            MoveDown => MoveUp,
            ActiveFlipX => ActiveFlipX,
            ActiveFlipY => ActiveFlipY,
            ActiveRotateCW => ActiveRotateCCW,
            ActiveRotateCCW => ActiveRotateCW,
        }
    }
}

pub fn handle_puzzle_action_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<PuzzleAction>,
    mut puzzles: Query<&mut Puzzle>,
    mut transforms: Query<&mut Transform>,
    mut actives: Query<&mut Active>,
) {
    use PuzzleAction::*;
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let size = puzzle.tiles.size();
        let prev_active_entity = {
            let active = puzzle.active;
            puzzle.get_tile_entity(active)
        };
        match event {
            MoveLeft | MoveRight | MoveUp | MoveDown => {
                let (entity, destination) = puzzle.apply_move_event(*event);
                if let Some(entity) = entity {
                    let mut tile_transform = transforms.get_mut(entity).expect("Oops");
                    tile_transform.translation =
                        tile_translation_from_position(destination, puzzle.tiles.size());
                }
            }
            ActiveFlipX | ActiveFlipY => {
                if let Some(tile) = puzzle.get_active_tile_mut() {
                    match event {
                        ActiveFlipX => tile.flip_x(),
                        ActiveFlipY => tile.flip_y(),
                        _ => panic!(),
                    }
                    if let Some(entity) = tile.entity {
                        let mesh = tile.compute_mesh(size);
                        commands.entity(entity).insert(meshes.add(mesh));
                        // Some Flipping state are replaced by a rotation
                        let mut transform =
                            transforms.get_mut(entity).expect("Tile has no transform");
                        transform.rotation = tile.compute_rotation();
                    }
                }
            }
            ActiveRotateCW | ActiveRotateCCW => {
                if let Some(tile) = puzzle.get_active_tile_mut() {
                    match event {
                        ActiveRotateCW => tile.rotate_cw(),
                        ActiveRotateCCW => tile.rotate_ccw(),
                        _ => panic!(),
                    }
                    if let Some(entity) = tile.entity {
                        let mut transform =
                            transforms.get_mut(entity).expect("Tile has no transform");
                        transform.rotation = tile.compute_rotation();
                    }
                }
            }
        }
        let new_active_entity = {
            let active = puzzle.active;
            puzzle.get_tile_entity(active)
        };
        if prev_active_entity != new_active_entity {
            if let Some(entity) = prev_active_entity {
                actives
                    .get_mut(entity)
                    .expect("A Tile Entity does not contains the Active component")
                    .0 = false;
            }
            if let Some(entity) = new_active_entity {
                actives
                    .get_mut(entity)
                    .expect("A Tile Entity does not contains the Active component")
                    .0 = true;
            }
        }
        if puzzle.is_solved() {
            println!("SOLVED");
        }
    }
}

pub fn active_tile(
    mut tiles: Query<(&mut Transform, &Active), Changed<Active>>,
    puzzle_assets: Option<Res<PuzzleAssets>>,
) {
    for (mut transform, active) in tiles.iter_mut() {
        let puzzle_assets = puzzle_assets
            .as_ref()
            .expect("No PuzzleAssets resource while tile entities with Active exists");
        if active.0 {
            transform.scale = puzzle_assets.active_tile_scale;
        } else {
            transform.scale = puzzle_assets.tile_scale;
        };
    }
}
