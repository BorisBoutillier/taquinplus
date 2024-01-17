use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Duration;

use crate::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::lens::TransformRotationLens;
use bevy_tweening::lens::TransformScaleLens;
use grid::Grid;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;

mod spawn;
pub use spawn::*;
// Coordinate for tile in the puzzle
// .0 is the row
// .1 is the column
pub type Coord = (usize, usize);

pub fn dist_of_1(p1: Coord, p2: Coord) -> bool {
    [(0, 1), (1, 0)].contains(&(
        (p1.0 as i32 - p2.0 as i32).abs(),
        (p1.1 as i32 - p2.1 as i32).abs(),
    ))
}
// Tag component for the parent of all solution tiles
#[derive(Component)]
pub struct PuzzleSolution;

// Tag component for the parent of all the current tiles
#[derive(Component)]
pub struct PuzzleTiles;

// Defines is this Tile entity is the Active one
#[derive(Component)]
pub struct Active(bool);

#[derive(Component)]
pub struct PuzzleAssets {
    // Default scale used by each tile
    tile_scale: Vec3,
    // scale used by each tile when the puzzle is solved
    solved_tile_scale: Vec3,
    // Color of the tile outline when the tile is both misplaced and misoriented
    outline_color_misplaced_misoriented: Color,
    // Color of the tile outline when the tile is misplaced
    outline_color_misplaced: Color,
    // Color of the tile outline when the tile is misoriented
    outline_color_misoriented: Color,
    // Color of the tile outline when the tile is the active tile
    outline_color_active: Color,
    // Material containing icon for most Puzzle Actions
    action_tip_materials: HashMap<PuzzleAction, Handle<StandardMaterial>>,
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
    pub action_tip_entity: Option<Entity>,
    pub actions_count: usize,
}
impl Puzzle {
    pub fn new(
        image: Handle<Image>,
        width: usize,
        height: usize,
        random_hole: bool,
        mut rng: impl RngCore,
    ) -> Self {
        let hole = if random_hole {
            (rng.gen_range(0..height), rng.gen_range(0..width))
        } else {
            (0, width - 1)
        };
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
            action_tip_entity: None,
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
            MoveActive => position = self.active,
            _ => panic!("Not a Move event: {:?}", event),
        }
        if !dist_of_1(self.hole, position) {
            (None, self.hole, self.hole)
        } else if let Some(tile) = self.tiles.get_mut(position.0, position.1).unwrap().take() {
            let destination = self.hole;
            self.active = destination;
            let entity = tile.entity;
            self.tiles[destination] = Some(tile);
            self.hole = position;
            (entity, destination, position)
        } else {
            (None, self.hole, self.hole)
        }
    }
    pub fn convert_action(&self, action: PuzzleAction) -> PuzzleAction {
        use PuzzleAction::*;
        if action == MoveActive {
            match (
                (self.hole.0 as i32 - self.active.0 as i32),
                (self.hole.1 as i32 - self.active.1 as i32),
            ) {
                (0, 1) => MoveRight,
                (0, -1) => MoveLeft,
                (1, 0) => MoveUp,
                (-1, 0) => MoveDown,
                _ => NoAction,
            }
        } else {
            action
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
    pub fn set_active_entity(&mut self, ref_entity: &Entity) {
        if let Some(coord) = self
            .tiles
            .indexed_iter()
            .filter_map(|(coord, tile)| {
                tile.as_ref()
                    .and_then(|tile| tile.entity)
                    .map(|entity| (coord, entity))
            })
            .find_map(|(coord, entity)| (&entity == ref_entity).then_some(coord))
        {
            self.active = coord;
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
                            (true, true) => assets.outline_color_misplaced_misoriented,
                            (true, false) => assets.outline_color_misplaced,
                            (false, true) => assets.outline_color_misoriented,
                            _ => assets.outline_color_active,
                        };
                        outline.width = if show_active { 1.0 } else { 3.0 };
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
        0.0,
    )
}

#[derive(Component)]
pub struct ActionTip;
#[derive(Component)]
pub struct ActionTipIcon;

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuzzleAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveActive,
    MoveActiveLeft,
    MoveActiveRight,
    MoveActiveUp,
    MoveActiveDown,
    SetActive(Entity),
    #[allow(dead_code)]
    ActiveFlipX,
    ActiveFlipY,
    ActiveRotateCW,
    ActiveRotateCCW,
    NoAction,
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
            ActiveRotateCW => ActiveRotateCCW,
            ActiveRotateCCW => ActiveRotateCW,
            _ => *self,
        }
    }
}

pub fn handle_puzzle_action_events(
    mut events: EventReader<PuzzleAction>,
    mut puzzle: Query<(&mut Puzzle, &PuzzleAssets)>,
    mut transforms: Query<&mut Transform>,
    mut action_tip_visibility: Query<&mut Visibility, With<ActionTip>>,
    mut tile_animations: Query<&mut TileAnimation>,
    mut outlines: Query<&mut OutlineVolume>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use PuzzleAction::*;
    for event in events.read() {
        if let Ok((mut puzzle, puzzle_assets)) = puzzle.get_single_mut() {
            if !puzzle.is_solved {
                match event {
                    NoAction => (),
                    MoveLeft | MoveRight | MoveUp | MoveDown | MoveActive => {
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
                            for mut visibility in action_tip_visibility.iter_mut() {
                                *visibility = Visibility::Hidden;
                            }
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
                    SetActive(entity) => {
                        puzzle.set_active_entity(entity);
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
                                let mut tile_animation =
                                    tile_animations.get_mut(entity).expect("Oops");
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
                                let mut tile_animation =
                                    tile_animations.get_mut(entity).expect("Oops");
                                tile_animation.push_transform_tween(tween);
                            }
                        }
                    }
                }
                puzzle.compute_solved();
                if puzzle.is_solved {
                    println!("SOLVED in {} actions", puzzle.actions_count);
                    next_state.set(GameState::PuzzleSolved);
                }
                puzzle.show_outlines(&mut outlines, puzzle_assets);
            }
        }
    }
}

pub fn show_full_puzzle(
    mut commands: Commands,
    // Restrict to Changed Puzzle, so that full size animation only happens once.
    // In particular we don't want to re-animate on called after a Menu::Show.
    puzzle: Query<(&Puzzle, &PuzzleAssets), Changed<Puzzle>>,
    mut action_tip_visibility: Query<&mut Visibility, With<ActionTip>>,
    mut tile_animations: Query<&mut TileAnimation>,
    mut outlines: Query<&mut OutlineVolume>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Hide all action tips, and all outlines
    for mut visibility in action_tip_visibility.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    for mut outline in outlines.iter_mut() {
        outline.visible = false;
    }
    // Animate all puzzle to reach their full size
    // Change the 'hole' tile to contain its image part.
    if let Ok((puzzle, puzzle_assets)) = puzzle.get_single() {
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
        if let Some(entity) = puzzle.hole_entity {
            let final_hole_mesh =
                meshes.add(compute_tile_mesh(puzzle.size(), puzzle.hole, false, false));
            let final_hole_material = materials.add(StandardMaterial {
                base_color_texture: Some(puzzle.image.clone()),
                reflectance: 0.0,
                ..default()
            });
            commands
                .entity(entity)
                .insert(final_hole_mesh)
                .insert(final_hole_material);
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

pub fn tile_on_click(
    event: Listener<Pointer<Click>>,
    mut puzzle_action_events: EventWriter<PuzzleAction>,
    transforms: Query<&GlobalTransform>,
) {
    let tile_global_transform = transforms
        .get(event.target)
        .expect("No Transform for Tile")
        .compute_transform();
    // local_translation will have a [-0.5,0.5] range for x and y
    let local_translation = (event.hit.position.unwrap() - tile_global_transform.translation)
        / tile_global_transform.scale;
    puzzle_action_events.send(action_from_tip_local(
        local_translation.x,
        local_translation.y,
    ));
}

pub fn tile_on_moving_over(
    event: Listener<Pointer<Move>>,
    mut puzzle_action_events: EventWriter<PuzzleAction>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    mut action_tip: Query<(Entity, &mut Visibility), With<ActionTip>>,
    puzzle: Query<(&Puzzle, &PuzzleAssets)>,
    mut action_tip_icon_material: Query<&mut Handle<StandardMaterial>, With<ActionTipIcon>>,
) {
    puzzle_action_events.send(PuzzleAction::SetActive(event.target));
    let tile_transform = *transforms.get(event.target).expect("No transform for tile");
    if let Ok((entity, mut action_tip)) = action_tip.get_single_mut() {
        *action_tip = Visibility::Inherited;
        let mut action_transform = transforms
            .get_mut(entity)
            .expect("No transform for ActionTip");
        action_transform.translation = tile_transform.translation;
    }
    // local_translation will have a [-0.5,0.5] range for x and y
    let tile_global_transform = global_transforms
        .get(event.target)
        .expect("No Transform for Tile")
        .compute_transform();
    // local_translation will have a [-0.5,0.5] range for x and y
    let local_translation = (event.hit.position.unwrap() - tile_global_transform.translation)
        / tile_global_transform.scale;
    let (puzzle, puzzle_assets) = puzzle.single();
    let action = puzzle.convert_action(action_from_tip_local(
        local_translation.x,
        local_translation.y,
    ));
    let mut action_tip_icon_material = action_tip_icon_material.single_mut();
    *action_tip_icon_material = puzzle_assets
        .action_tip_materials
        .get(&action)
        .unwrap_or_else(|| panic!("No icon for action {action:?}"))
        .clone();
}

pub fn action_from_tip_local(x: f32, y: f32) -> PuzzleAction {
    assert!((-0.5..=0.5).contains(&x), "Unexpected x:{x}");
    assert!((-0.5..=0.5).contains(&y), "Unexpected y:{y}");
    match ((x < -0.3, x > 0.3), (y < -0.3, y > 0.3)) {
        ((true, false), (true, false)) => PuzzleAction::ActiveRotateCCW, // Left Down
        ((true, false), (false, false)) => PuzzleAction::ActiveFlipX,    // Left Middle
        ((true, false), (false, true)) => PuzzleAction::ActiveRotateCCW, // Left Up
        ((false, false), (true, false)) => PuzzleAction::ActiveFlipY,    // Middle Down
        ((false, false), (false, false)) => PuzzleAction::MoveActive,    // Middle Middle
        ((false, false), (false, true)) => PuzzleAction::ActiveFlipY,    // Middle Up
        ((false, true), (true, false)) => PuzzleAction::ActiveRotateCW,  // Right Down
        ((false, true), (false, false)) => PuzzleAction::ActiveFlipX,    // Right Middle
        ((false, true), (false, true)) => PuzzleAction::ActiveRotateCW,  // Right Up
        _ => panic!("What ? x:{}, y:{}", x, y),
    }
}

const PUZZLE_SOLVED_EXIT: [KeyCode; 3] = [KeyCode::Return, KeyCode::Space, KeyCode::Escape];

pub fn puzzle_solving_interaction(
    mut puzzle_solution: Query<&mut Visibility, (With<PuzzleSolution>, Without<PuzzleTiles>)>,
    mut puzzle_tiles: Query<&mut Visibility, With<PuzzleTiles>>,
    mut puzzle: Query<(&mut Puzzle, &PuzzleAssets)>,
    mut outlines: Query<&mut OutlineVolume>,
    input: Res<Input<KeyCode>>,
    mouse_button: Res<Input<MouseButton>>,
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
    if input.just_pressed(KeyCode::Escape) || mouse_button.just_pressed(MouseButton::Right) {
        next_gamestate.set(GameState::Menu);
    }
}
pub fn puzzle_solved_interaction(
    input: Res<Input<KeyCode>>,
    mouse_button: Res<Input<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.any_just_pressed(PUZZLE_SOLVED_EXIT)
        || mouse_button.any_just_pressed([MouseButton::Left, MouseButton::Right])
    {
        next_state.set(GameState::Menu);
    }
}
