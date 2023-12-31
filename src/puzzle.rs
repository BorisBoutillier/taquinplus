use crate::prelude::*;

#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub current_tile: Option<Entity>,
    pub hole: Option<(isize, isize)>,
    pub tiles: Vec<Vec<Option<Entity>>>,
    pub len_x: isize,
    pub len_y: isize,
}
impl Puzzle {
    pub fn new(image: Handle<Image>, len_x: isize, len_y: isize) -> Self {
        Puzzle {
            image,
            current_tile: None,
            hole: None,
            tiles: vec![],
            len_x,
            len_y,
        }
    }
    pub fn spawn(
        mut self,
        commands: &mut Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let hole = (4, 0);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(self.image.clone()),
            reflectance: 0.0,
            ..default()
        });
        let tile_scale = 0.93 / (self.len_x.max(self.len_y) as f32);
        let tile_tf = Transform::from_scale(Vec3::new(tile_scale, tile_scale, 5.));
        let tiles: Vec<Vec<Option<Entity>>> = (0..self.len_x)
            .map(|x| {
                (0..self.len_y)
                    .map(|y| {
                        if (x, y) == hole {
                            None
                        } else {
                            Some(
                                commands
                                    .spawn(PbrBundle {
                                        material: material.clone(),
                                        transform: tile_tf,
                                        ..default()
                                    })
                                    .insert(Piece {
                                        x,
                                        y,
                                        len_x: 5,
                                        len_y: 5,
                                    })
                                    .insert(Position {
                                        x,
                                        y,
                                        len_x: self.len_x,
                                        len_y: self.len_y,
                                    })
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
        self.tiles = tiles;
        self.hole = Some(hole);
        commands
            .spawn(SpatialBundle::default())
            .push_children(
                &self
                    .tiles
                    .iter()
                    .flat_map(|tiles| tiles.iter().filter_map(|tile| *tile))
                    .collect::<Vec<_>>(),
            )
            .insert(self);
    }
    pub fn move_to_hole(&mut self, x: isize, y: isize, positions: &mut Query<&mut Position>) {
        let (hole_x, hole_y) = self.hole.unwrap();
        if let Some(tile) = self.tiles[x as usize][y as usize].take() {
            let mut tile_position = positions.get_mut(tile).expect("Oops");
            tile_position.x = hole_x;
            tile_position.y = hole_y;
            self.tiles[hole_x as usize][hole_y as usize] = Some(tile);
            self.hole = Some((x, y));
            self.current_tile = Some(tile);
        }
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
    mut positions: Query<&mut Position>,
) {
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let (mut new_hole_x, mut new_hole_y) = puzzle.hole.unwrap();
        match event {
            PuzzleMoveEvent::MoveLeft => new_hole_x = (new_hole_x + 1).min(puzzle.len_x - 1),
            PuzzleMoveEvent::MoveRight => new_hole_x = (new_hole_x - 1).max(0),
            PuzzleMoveEvent::MoveUp => new_hole_y = (new_hole_y - 1).max(0),
            PuzzleMoveEvent::MoveDown => new_hole_y = (new_hole_y + 1).min(puzzle.len_y - 1),
            PuzzleMoveEvent::Move(tile) => {
                let position = positions
                    .get(*tile)
                    .expect("Move call with an Entity that does not have a Position");
                match (
                    (new_hole_x - position.x).abs(),
                    (new_hole_y - position.y).abs(),
                ) {
                    (0, 1) | (1, 0) => {
                        new_hole_x = position.x;
                        new_hole_y = position.y;
                    }
                    _ => (),
                }
            }
        }
        puzzle.move_to_hole(new_hole_x, new_hole_y, &mut positions);
    }
}
