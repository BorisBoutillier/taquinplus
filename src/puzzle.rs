use crate::prelude::*;
use grid::Grid;
#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub active: (usize, usize),
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
            len_x: width as isize,
            len_y: height as isize,
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
                            mesh: meshes.add(tile.compute_mesh(size.0, size.1)),
                            material: material.clone(),
                            transform: tile_transform.with_translation(
                                Self::tile_translation_from_position((y, x), size),
                            ),
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
    pub fn move_to_hole(&mut self, y: usize, x: usize, transforms: &mut Query<&mut Transform>) {
        if let Some(tile) = self.tiles.get_mut(y, x).unwrap().take() {
            if let Some(entity) = tile.entity {
                let mut tile_transform = transforms.get_mut(entity).expect("Oops");
                tile_transform.translation =
                    Self::tile_translation_from_position(self.hole, self.tiles.size());
                self.active = self.hole;
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
pub enum PuzzleActionEvent {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    #[allow(dead_code)]
    Move(Entity),
    ActiveFlipX,
    ActiveFlipY,
    ActiveRotateCW,
    ActiveRotateCCW,
}

pub fn handle_action_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<PuzzleActionEvent>,
    mut puzzles: Query<&mut Puzzle>,
    mut transforms: Query<&mut Transform>,
) {
    use PuzzleActionEvent::*;
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let height = puzzle.len_y as usize;
        let width = puzzle.len_x as usize;
        let active = puzzle.active;
        match event {
            MoveLeft | MoveRight | MoveUp | MoveDown | Move(_) => {
                let (mut new_hole_y, mut new_hole_x) = puzzle.hole;
                match event {
                    MoveLeft => new_hole_x = (new_hole_x + 1).min(puzzle.len_x as usize - 1),
                    MoveRight => new_hole_x = new_hole_x.max(1) - 1,
                    MoveUp => new_hole_y = new_hole_y.max(1) - 1,
                    MoveDown => new_hole_y = (new_hole_y + 1).min(puzzle.len_y as usize - 1),
                    _ => panic!(),
                }
                puzzle.move_to_hole(new_hole_y, new_hole_x, &mut transforms);
            }
            ActiveFlipX | ActiveFlipY => {
                if let Some(tile) = puzzle.tiles.get_mut(active.0, active.1).unwrap() {
                    match event {
                        ActiveFlipX => tile.flip_x(),
                        ActiveFlipY => tile.flip_y(),
                        _ => panic!(),
                    }
                    if let Some(entity) = tile.entity {
                        let mesh = tile.compute_mesh(height, width);
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
