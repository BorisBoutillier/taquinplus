use crate::prelude::*;
use grid::Grid;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;

// Coordinate for tile in the puzzle
// .0 is the row
// .1 is the column
type Coord = (usize, usize);

#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub active: Coord,
    pub hole: Coord,
    pub tiles: Grid<Option<Tile>>,
    pub len_x: usize,
    pub len_y: usize,
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
            len_x: width,
            len_y: height,
        }
    }
    pub fn shuffle(
        &mut self,
        n_moves: usize,
        flip_pct: f64,
        rotation_pct: f64,
        mut rng: impl RngCore,
    ) {
        for tile in self.tiles.iter_mut().filter_map(|tile| tile.as_mut()) {
            if rng.gen_bool(flip_pct) {
                let what = rng.gen_range(1..=3u8);
                if what & 1 == 1 {
                    tile.flip_x();
                }
                if what & 2 == 2 {
                    tile.flip_y();
                }
            }
            if rng.gen_bool(rotation_pct) {
                for _ in 0..(rng.gen_range(1..=3u8)) {
                    tile.rotate_cw();
                }
            }
        }
        let mut reverse_move = None;
        for _ in 0..n_moves {
            let mut possible_moves = self.get_valid_moves();
            possible_moves.retain(|action| Some(*action) != reverse_move);
            let action = possible_moves
                .choose(&mut rng)
                .expect("No possible move found");
            reverse_move = Some(action.reverse());
            self.apply_move_event(*action);
        }
    }
    pub fn spawn(
        mut self,
        commands: &mut Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(self.image.clone()),
            reflectance: 0.0,
            ..default()
        });
        let tile_scale = 0.93 / (self.len_x.max(self.len_y) as f32);
        let tile_transform = Transform::from_scale(Vec3::new(tile_scale, tile_scale, 5.));
        let size = self.tiles.size();
        self.tiles.indexed_iter_mut().for_each(|((y, x), tile)| {
            if let Some(tile) = tile.as_mut() {
                tile.entity = Some(
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(tile.compute_mesh(size)),
                            material: material.clone(),
                            transform: tile_transform
                                .with_translation(tile_translation_from_position((y, x), size)),
                            ..default()
                        })
                        .id(),
                );
            }
        });
        commands
            .spawn(SpatialBundle::default())
            .push_children(
                &self
                    .tiles
                    .iter()
                    .filter_map(|tile| tile.as_ref().and_then(|tile| tile.entity))
                    .collect::<Vec<_>>(),
            )
            .insert(self);
    }
    pub fn apply_move_event(&mut self, event: PuzzleAction) -> (Option<Entity>, Coord) {
        use PuzzleAction::*;
        let mut position = self.hole;
        let size = self.tiles.size();
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
        let size = self.tiles.size();
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
) {
    use PuzzleAction::*;
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let size = puzzle.tiles.size();
        let active = puzzle.active;
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
                if let Some(tile) = puzzle.tiles.get_mut(active.0, active.1).unwrap() {
                    match event {
                        ActiveFlipX => tile.flip_x(),
                        ActiveFlipY => tile.flip_y(),
                        _ => panic!(),
                    }
                    if let Some(entity) = tile.entity {
                        let mesh = tile.compute_mesh(size);
                        commands.entity(entity).insert(meshes.add(mesh));
                    }
                }
            }
            ActiveRotateCW | ActiveRotateCCW => {
                if let Some(tile) = puzzle.tiles.get_mut(active.0, active.1).unwrap() {
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
    }
}
