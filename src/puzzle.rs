use std::collections::VecDeque;
use std::time::Duration;

use crate::prelude::*;
use bevy::ecs::system::Command;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::lens::TransformRotationLens;
use bevy_tweening::lens::TransformScaleLens;
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
    // Default scale used by each tile
    tile_scale: Vec3,
    // scale used by each tile when the puzzle is solved
    solved_tile_scale: Vec3,
    // Color of the tile outline when the tile is misplaced
    outline_color_misplaced: Color,
    // Color of the tile outline when the tile is misoriented
    outline_color_misoriented: Color,
    // Color of the tile outline when the tile is the active tile
    outline_color_active: Color,
}

#[derive(Component)]
pub struct Puzzle {
    pub image: Handle<Image>,
    pub active: Coord,
    pub hole: Coord,
    pub tiles: Grid<Option<Tile>>,
    pub is_solved: bool,
    pub show_errors: bool,
    pub hole_entity: Option<Entity>,
    pub actions_count: usize,
}
impl Puzzle {
    pub fn new(image: Handle<Image>, width: usize, height: usize) -> Self {
        let hole = (0, width - 1);
        let puzzle_size = (height, width);
        let tiles = Grid::from_vec(
            (0..height)
                .flat_map(|y| {
                    (0..width)
                        .map(|x| {
                            if (y, x) == hole {
                                None
                            } else {
                                Some(Tile::new((y, x), puzzle_size))
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
            is_solved: false,
            show_errors: false,
            hole_entity: None,
            actions_count: 0,
        }
    }
    pub fn get_active_tile_mut(&mut self) -> &mut Option<Tile> {
        self.tiles
            .get_mut(self.active.0, self.active.1)
            .expect("Invalid Active tile")
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
        self.is_solved = false;
    }
    pub fn compute_solved(&mut self) {
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
        self.is_solved = incorrect_placement == 0 && incorrect_flip == 0 && incorrect_rotation == 0;
    }
    pub fn apply_move_event(&mut self, event: PuzzleAction) -> (Option<Entity>, Coord, Coord) {
        use PuzzleAction::*;
        if self.is_solved {
            warn!("Move event while puzzle is solved");
        }
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
            (entity, destination, position)
        } else {
            (None, position, position)
        }
    }
    pub fn apply_move_active_event(&mut self, event: PuzzleAction) {
        use PuzzleAction::*;
        if self.is_solved {
            warn!("MoveActive event while puzzle is solved");
        }
        let mut position = self.active;
        let size = self.size();
        match event {
            MoveActiveLeft => position.1 = position.1.max(1) - 1,
            MoveActiveRight => position.1 = (position.1 + 1).min(size.1 - 1),
            MoveActiveUp => position.0 = (position.0 + 1).min(size.0 - 1),
            MoveActiveDown => position.0 = position.0.max(1) - 1,
            _ => panic!("Not a MoveActive event: {:?}", event),
        }
        self.active = position;
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
    pub fn show_outlines(
        &mut self,
        outlines: &mut Query<&mut OutlineVolume>,
        assets: &PuzzleAssets,
    ) {
        self.tiles.indexed_iter_mut().for_each(|(index, tile)| {
            if let Some(tile) = tile {
                if let Some(entity) = tile.entity {
                    if let Ok(mut outline) = outlines.get_mut(entity) {
                        let show_misplaced = self.show_errors && index != tile.position;
                        let show_misoriented = self.show_errors && !tile.is_correctly_oriented();
                        let show_active =
                            self.active == index && !show_misoriented && !show_misplaced;
                        outline.visible =
                            !self.is_solved && (show_active || show_misplaced || show_misoriented);
                        outline.colour = match (show_misplaced, show_misoriented) {
                            (true, _) => assets.outline_color_misplaced,
                            (_, true) => assets.outline_color_misoriented,
                            _ => assets.outline_color_active,
                        };
                        outline.width = if show_active { 1.0 } else { 2.0 };
                    }
                }
            }
        });
        if let Some(entity) = self.hole_entity {
            if let Ok(mut outline) = outlines.get_mut(entity) {
                outline.visible = self.hole == self.active;
            }
        }
    }
}
fn tile_translation_from_position(position: (usize, usize), size: (usize, usize)) -> Vec3 {
    Vec3::new(
        (2 * position.1 as isize + 1 - size.1 as isize) as f32 / (2 * size.1) as f32,
        (2 * position.0 as isize + 1 - size.0 as isize) as f32 / (2 * size.0) as f32,
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
        let tile_scale = {
            let scale = 0.93 / (size.0.max(size.1) as f32);
            Vec3::new(scale, scale, 5.)
        };
        let solved_tile_scale = {
            let scale = 1. / (size.0.max(size.1) as f32);
            Vec3::new(scale, scale, 5.)
        };
        let tile_transform = Transform::from_scale(tile_scale);
        let mut solution_tiles = vec![];
        self.tiles.indexed_iter_mut().for_each(|(index, tile)| {
            if let Some(tile) = tile.as_mut() {
                let mesh = {
                    let mut meshes = world
                        .get_resource_mut::<Assets<Mesh>>()
                        .expect("No Resource Assets<Mesh>");

                    meshes.add(tile.compute_mesh())
                };
                tile.entity = Some(
                    world
                        .spawn(PbrBundle {
                            material: tile_material.clone(),
                            mesh,
                            transform: tile_transform
                                .with_rotation(tile.compute_rotation())
                                .with_translation(tile_translation_from_position(index, size)),
                            ..default()
                        })
                        .insert(TileAnimationBundle::default())
                        .insert(Name::new(format!("Tile_Ref_{}x{}", index.1, index.0)))
                        .insert(OutlineBundle {
                            outline: OutlineVolume {
                                visible: false,
                                width: 2.0,
                                colour: Color::WHITE,
                            },
                            ..default()
                        })
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
        // Spawn the hole entity
        let hole_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("No Resource Assets<StandardMaterial>");
            materials.add(Color::rgba(1.0, 1.0, 1.0, 0.0).into())
        };
        let hole_mesh = {
            let mut meshes = world
                .get_resource_mut::<Assets<Mesh>>()
                .expect("No Resource Assets<Mesh>");

            meshes.add(Mesh::from(shape::Cube::new(1.0)))
        };
        let hole_entity = world
            .spawn(PbrBundle {
                material: hole_material,
                mesh: hole_mesh,
                transform: tile_transform
                    .with_translation(tile_translation_from_position(self.hole, size)),
                ..default()
            })
            .insert(Name::new(format!(
                "Hole_Ref_{}x{}",
                self.hole.1, self.hole.0
            )))
            .insert(TileAnimationBundle::default())
            .insert(OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: Color::WHITE,
                },
                ..default()
            })
            .id();
        self.hole_entity = Some(hole_entity);
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
        let puzzle_tiles = world
            .spawn(SpatialBundle::default())
            .push_children(
                &self
                    .tiles
                    .iter()
                    .filter_map(|tile| tile.as_ref().and_then(|tile| tile.entity))
                    .collect::<Vec<_>>(),
            )
            .add_child(hole_entity)
            .insert(Name::new("Tiles"))
            .insert(PuzzleTiles)
            .id();

        // Spawn the puzzle main entity, with solution and tiles children
        world
            .spawn(SpatialBundle::default())
            .insert(Name::new("Puzzle"))
            .insert(self)
            .add_child(puzzle_solution)
            .add_child(puzzle_tiles);
        // Create the resource containing all the needed asset handles for the Puzzle
        world.insert_resource(PuzzleAssets {
            tile_scale,
            solved_tile_scale,
            outline_color_active: Color::WHITE,
            outline_color_misoriented: Color::ORANGE,
            outline_color_misplaced: Color::RED,
        });
    }
}

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveActiveLeft,
    MoveActiveRight,
    MoveActiveUp,
    MoveActiveDown,
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
            MoveActiveLeft => MoveActiveRight,
            MoveActiveRight => MoveActiveLeft,
            MoveActiveUp => MoveActiveDown,
            MoveActiveDown => MoveActiveUp,
            ActiveFlipX => ActiveFlipX,
            ActiveFlipY => ActiveFlipY,
            ActiveRotateCW => ActiveRotateCCW,
            ActiveRotateCCW => ActiveRotateCW,
        }
    }
}

const ACTION_ANIMATION_DURATION: u64 = 150;
pub fn handle_puzzle_action_events(
    mut commands: Commands,
    mut events: EventReader<PuzzleAction>,
    mut puzzles: Query<&mut Puzzle>,
    mut transforms: Query<&mut Transform>,
    mut tile_animations: Query<&mut TileAnimation>,
    mut outlines: Query<&mut OutlineVolume>,
    puzzle_assets: Option<Res<PuzzleAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use PuzzleAction::*;
    for event in events.read() {
        let mut puzzle = puzzles.single_mut();
        let puzzle_assets = puzzle_assets
            .as_ref()
            .expect("No PuzzleAssets resource while tile entities with Active exists");
        if !puzzle.is_solved {
            match event {
                MoveLeft | MoveRight | MoveUp | MoveDown => {
                    let (entity, destination, source) = puzzle.apply_move_event(*event);
                    if let Some(entity) = entity {
                        let start_translation =
                            tile_translation_from_position(source, puzzle.size());
                        let end_translation =
                            tile_translation_from_position(destination, puzzle.size());
                        let tween = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_millis(ACTION_ANIMATION_DURATION),
                            TransformPositionLens {
                                start: start_translation,
                                end: end_translation,
                            },
                        );
                        // This action count should be on puzzle methods
                        puzzle.actions_count += 1;
                        let mut tile_animation = tile_animations.get_mut(entity).expect("Oops");
                        tile_animation.push_transform_tween(tween);
                    }
                    let hole = puzzle.hole;
                    let size = puzzle.size();
                    if let Some(hole_entity) = puzzle.hole_entity {
                        let mut transform = transforms
                            .get_mut(hole_entity)
                            .expect("No Transform for the hole entity");
                        transform.translation = tile_translation_from_position(hole, size)
                    }
                }
                MoveActiveLeft | MoveActiveRight | MoveActiveUp | MoveActiveDown => {
                    puzzle.apply_move_active_event(*event);
                }
                ActiveFlipX | ActiveFlipY => {
                    // TODO: this effective flip and event conversion should be a puzzle method
                    // Only handling here a returned tile_entity and returned effective event
                    // TODO: This action count should be on puzzle methods
                    if puzzle.active != puzzle.hole {
                        puzzle.actions_count += 1;
                    }
                    if let Some(tile) = puzzle.get_active_tile_mut() {
                        // Convert the X/Y user axis to the local tile axis, based on tile rotation
                        let local_event = {
                            use CwRotation::*;
                            match tile.rotation {
                                R0 | R180 => event,
                                R90 | R270 => match event {
                                    ActiveFlipX => &ActiveFlipY,
                                    ActiveFlipY => &ActiveFlipX,
                                    _ => panic!(),
                                },
                            }
                        };
                        match local_event {
                            ActiveFlipX => tile.flip_x(),
                            ActiveFlipY => tile.flip_y(),
                            _ => panic!(),
                        }
                        if let Some(entity) = tile.entity {
                            let tween = Tween::new(
                                EaseFunction::QuadraticInOut,
                                Duration::from_millis(ACTION_ANIMATION_DURATION),
                                match local_event {
                                    ActiveFlipX => MeshFlippingLens::new_flip_x(tile.clone()),
                                    ActiveFlipY => MeshFlippingLens::new_flip_y(tile.clone()),
                                    _ => panic!(),
                                },
                            );
                            let mut tile_animation = tile_animations.get_mut(entity).expect("Oops");
                            tile_animation.push_mesh_tween(tween);
                        }
                    }
                }
                ActiveRotateCW | ActiveRotateCCW => {
                    // TODO: this rotationd event should be a puzzle method
                    // Only handling here a returned tile_entity
                    // TODO: This action count should be on puzzle methods
                    if puzzle.active != puzzle.hole {
                        puzzle.actions_count += 1;
                    }
                    if let Some(tile) = puzzle.get_active_tile_mut() {
                        let start_rotation = tile.compute_rotation();
                        match event {
                            ActiveRotateCW => tile.rotate_cw(),
                            ActiveRotateCCW => tile.rotate_ccw(),
                            _ => panic!(),
                        }
                        if let Some(entity) = tile.entity {
                            let end_rotation = tile.compute_rotation();
                            let tween = Tween::new(
                                EaseFunction::QuadraticInOut,
                                Duration::from_millis(ACTION_ANIMATION_DURATION),
                                TransformRotationLens {
                                    start: start_rotation,
                                    end: end_rotation,
                                },
                            );
                            let mut tile_animation = tile_animations.get_mut(entity).expect("Oops");
                            tile_animation.push_transform_tween(tween);
                        }
                    }
                }
            }
            puzzle.compute_solved();
            if puzzle.is_solved {
                println!("SOLVED in {} actions", puzzle.actions_count);
                for tile in puzzle.tiles.iter().filter_map(|tile| tile.as_ref()) {
                    if let Some(entity) = tile.entity {
                        let tween = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_millis(500),
                            TransformScaleLens {
                                start: puzzle_assets.tile_scale,
                                end: puzzle_assets.solved_tile_scale,
                            },
                        );
                        let mut tile_animation = tile_animations.get_mut(entity).expect("Oops");
                        tile_animation.push_transform_tween(tween);
                    }
                }
                let final_mesh =
                    meshes.add(compute_tile_mesh(puzzle.size(), puzzle.hole, false, false));
                let final_material = materials.add(StandardMaterial {
                    base_color_texture: Some(puzzle.image.clone()),
                    reflectance: 0.0,
                    ..default()
                });
                if let Some(entity) = puzzle.hole_entity {
                    commands
                        .entity(entity)
                        .insert(final_mesh)
                        .insert(final_material);
                    let tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(500),
                        TransformScaleLens {
                            start: Vec3::new(0.0, 0.0, puzzle_assets.tile_scale.z),
                            end: puzzle_assets.solved_tile_scale,
                        },
                    );
                    let mut tile_animation = tile_animations.get_mut(entity).expect("Oops");
                    tile_animation.push_transform_tween(tween);
                }
            }
            puzzle.show_outlines(&mut outlines, puzzle_assets)
        }
    }
}

#[derive(Component)]
pub struct TileAnimation {
    queue: VecDeque<(BoxedTweenable<Transform>, BoxedTweenable<Mesh>)>,
}
impl TileAnimation {
    pub fn push_transform_tween(&mut self, tween: impl Tweenable<Transform> + 'static) {
        let duration = tween.duration();
        self.queue
            .push_back((Box::new(tween), Box::new(Delay::new(duration))))
    }
    pub fn push_mesh_tween(&mut self, tween: impl Tweenable<Mesh> + 'static) {
        let duration = tween.duration();
        self.queue
            .push_back((Box::new(Delay::new(duration)), Box::new(tween)))
    }
}
#[derive(Bundle)]
pub struct TileAnimationBundle {
    tile_animator: TileAnimation,
    transform_animator: Animator<Transform>,
    mesh_animator: AssetAnimator<Mesh>,
}
impl Default for TileAnimationBundle {
    fn default() -> Self {
        Self {
            tile_animator: TileAnimation {
                queue: VecDeque::new(),
            },
            transform_animator: Animator::new(Delay::new(Duration::from_millis(1))),
            mesh_animator: AssetAnimator::new(Delay::new(Duration::from_millis(1))),
        }
    }
}

pub fn tile_animation(
    mut animations: Query<(
        &mut TileAnimation,
        &mut Animator<Transform>,
        &mut AssetAnimator<Mesh>,
    )>,
) {
    for (mut tile_animation, mut transform_animator, mut mesh_animator) in animations.iter_mut() {
        if !tile_animation.queue.is_empty()
            && transform_animator.tweenable().progress() >= 1.0
            && mesh_animator.tweenable().progress() >= 1.0
        {
            let (transform_tween, mesh_tween) = tile_animation.queue.pop_front().unwrap();
            transform_animator.set_tweenable(Sequence::new([transform_tween]));
            mesh_animator.set_tweenable(Sequence::new([mesh_tween]));
        }
    }
}
