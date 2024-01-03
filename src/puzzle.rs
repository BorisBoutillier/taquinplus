use crate::prelude::*;
use grid::Grid;

#[derive(Default, Debug)]
pub struct Tile {
    // When spawned, Bevy entity associated to this tile
    entity: Option<Entity>,
}
#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub current_tile: Option<Entity>,
    pub hole: (usize, usize),
    pub tiles: Grid<Option<Tile>>,
    pub len_x: isize,
    pub len_y: isize,
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
                                Some(Tile::default())
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            width,
        );
        Puzzle {
            image,
            current_tile: None,
            hole,
            tiles,
            len_x: width as isize,
            len_y: height as isize,
        }
    }
    pub fn spawn(
        mut self,
        commands: &mut Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(self.image.clone()),
            reflectance: 0.0,
            ..default()
        });
        let tile_scale = 0.93 / (self.len_x.max(self.len_y) as f32);
        let tile_tf = Transform::from_scale(Vec3::new(tile_scale, tile_scale, 5.));
        let size = self.tiles.size();
        self.tiles.indexed_iter_mut().for_each(|((y, x), tile)| {
            if let Some(tile) = tile.as_mut() {
                tile.entity = Some(
                    commands
                        .spawn(PbrBundle {
                            material: material.clone(),
                            transform: tile_tf.with_translation(
                                Self::tile_translation_from_position((y, x), size),
                            ),
                            ..default()
                        })
                        .insert(Piece {
                            x: x as isize,
                            y: y as isize,
                            len_x: size.1 as isize,
                            len_y: size.0 as isize,
                        })
                        .insert(Flipped {
                            flipped_x: false,
                            flipped_y: false,
                        })
                        .insert(Rotated::default())
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
    pub fn move_to_hole(&mut self, y: usize, x: usize, transforms: &mut Query<&mut Transform>) {
        if let Some(tile) = self.tiles.get_mut(y, x).unwrap().take() {
            if let Some(entity) = tile.entity {
                let mut tile_transform = transforms.get_mut(entity).expect("Oops");
                tile_transform.translation =
                    Self::tile_translation_from_position(self.hole, self.tiles.size());
                self.current_tile = Some(entity);
            }
            self.tiles[self.hole] = Some(tile);
            self.hole = (y, x);
        }
    }
    fn tile_translation_from_position(position: (usize, usize), size: (usize, usize)) -> Vec3 {
        Vec3::new(
            (position.1 as isize - (size.1 as isize / 2)) as f32 / size.1 as f32,
            (position.0 as isize - (size.0 as isize / 2)) as f32 / size.0 as f32,
            0.,
        )
    }
}

#[derive(Event)]
pub enum PuzzleMoveEvent {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    #[allow(dead_code)]
    Move(Entity),
}

pub fn handle_action_events(
    mut events: EventReader<PuzzleMoveEvent>,
    mut puzzles: Query<&mut Puzzle>,
    mut transforms: Query<&mut Transform>,
) {
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let (mut new_hole_y, mut new_hole_x) = puzzle.hole;
        match event {
            PuzzleMoveEvent::MoveLeft => {
                new_hole_x = (new_hole_x + 1).min(puzzle.len_x as usize - 1)
            }
            PuzzleMoveEvent::MoveRight => new_hole_x = new_hole_x.max(1) - 1,
            PuzzleMoveEvent::MoveUp => new_hole_y = new_hole_y.max(1) - 1,
            PuzzleMoveEvent::MoveDown => {
                new_hole_y = (new_hole_y + 1).min(puzzle.len_y as usize - 1)
            }
            PuzzleMoveEvent::Move(_tile) => {
                // Need the Grid to more conveniently find back position of the tile in the puzzle
                panic!();
            }
        }
        puzzle.move_to_hole(new_hole_y, new_hole_x, &mut transforms);
    }
}
