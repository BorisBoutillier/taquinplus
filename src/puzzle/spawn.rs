use std::collections::HashMap;

use crate::prelude::*;

use super::tile_translation_from_position;

pub fn spawn_puzzle_entities(
    mut commands: Commands,
    mut puzzle: Query<(Entity, &mut Puzzle), Without<Visibility>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    for (puzzle_entity, mut puzzle) in puzzle.iter_mut() {
        let tile_material = materials.add(StandardMaterial {
            base_color_texture: Some(puzzle.image.clone()),
            reflectance: 0.0,
            ..default()
        });
        let size = puzzle.size();
        let tile_scale = {
            let scale = TILE_OCCUPANCY / (size.0.max(size.1) as f32);
            Vec3::new(scale, scale, 1.)
        };
        let solved_tile_scale = {
            let scale = 1. / (size.0.max(size.1) as f32);
            Vec3::new(scale, scale, 5.)
        };
        let tile_transform = Transform::from_scale(tile_scale);
        let mut solution_tiles = vec![];
        puzzle.tiles.indexed_iter_mut().for_each(|(index, tile)| {
            if let Some(tile) = tile.as_mut() {
                let mesh = meshes.add(tile.compute_mesh());
                tile.entity = Some(
                    commands
                        .spawn((
                            PbrBundle {
                                material: tile_material.clone(),
                                mesh,
                                transform: tile_transform
                                    .with_rotation(tile.compute_rotation())
                                    .with_translation(tile_translation_from_position(index, size)),
                                ..default()
                            },
                            TileAnimationBundle::default(),
                            Name::new(format!("Tile_Ref_{}x{}", index.1, index.0)),
                            OutlineBundle {
                                outline: OutlineVolume {
                                    visible: false,
                                    width: 2.0,
                                    colour: Color::WHITE,
                                },
                                ..default()
                            },
                            On::<Pointer<Move>>::run(tile_on_moving_over),
                            On::<Pointer<Click>>::run(tile_on_click),
                        ))
                        .id(),
                );
                // Duplicate the tile to add to the PuzzleSolution at the real tile position
                // Need a duplicate mesh, because it must not be flipped when the main tile is flipped
                let solution_mesh =
                    meshes.add(compute_tile_mesh(size, tile.position, false, false));
                solution_tiles.push(
                    commands
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
        let hole_material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.0));
        let hole_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let hole_entity = commands
            .spawn(PbrBundle {
                material: hole_material,
                mesh: hole_mesh,
                transform: tile_transform
                    .with_translation(tile_translation_from_position(puzzle.hole, size)),
                ..default()
            })
            .insert(Name::new(format!(
                "Hole_Ref_{}x{}",
                puzzle.hole.1, puzzle.hole.0
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
        puzzle.hole_entity = Some(hole_entity);
        // Action tip entity
        let action_tip_material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_GRID_ALPHA),
            base_color_texture: Some(asset_server.load("ActionTipFullGrid.png")),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let puzzle_action_tip = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0., Z_PUZZLE_ACTION_TIP)),
                    ..default()
                },
                Name::new("ActionTip"),
            ))
            .with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        visibility: Visibility::Hidden,
                        transform: tile_transform,
                        mesh: meshes.add(Mesh::from(Rectangle::new(1.0,1.0))),
                        material: action_tip_material,
                        ..default()
                    },
                    ActionTip,
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                    On::<Pointer<Out>>::run(
                        |_event: Listener<Pointer<Out>>,
                         mut action_tip: Query<
                            &mut Visibility,
                            With<ActionTip>,
                        >| {
                            if let Ok(mut action_tip) = action_tip.get_single_mut() {
                                *action_tip = Visibility::Hidden;
                            }
                        },
                )))
            .with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        transform: Transform::from_scale(Vec3::new(0.5,0.5,1.))
                            // Slightly on top of the action_tip_grid parent to avoid clipping 
                            .with_translation(Vec3::new(0.0,0.0,0.1)),
                        mesh: meshes.add(Mesh::from(Rectangle::new(1.0,1.0))),
                        ..default()
                    }, ActionTipIcon,Pickable::IGNORE)
            );});
            })
            .id();
        puzzle.action_tip_entity = Some(puzzle_action_tip);
        // Spawn the puzzle solution parent
        let puzzle_solution = commands
            .spawn(SpatialBundle {
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(Vec3::new(0., 0., Z_PUZZLE_SOLUTION)),
                ..default()
            })
            .push_children(&solution_tiles)
            .insert(Name::new("Solution"))
            .insert(PuzzleSolution)
            .id();
        // Spawn the puzzle tiles parent
        let puzzle_tiles = commands
            .spawn(SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., Z_PUZZLE_TILE)),
                ..default()
            })
            .push_children(
                &puzzle
                    .tiles
                    .iter()
                    .filter_map(|tile| tile.as_ref().and_then(|tile| tile.entity))
                    .collect::<Vec<_>>(),
            )
            .add_child(hole_entity)
            .insert(Name::new("Tiles"))
            .insert(PuzzleTiles)
            .id();

        let mut action_tip_materials = HashMap::new();
        // Action tip entity
        action_tip_materials.insert(
            PuzzleAction::ActiveFlipX,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconFlipX.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::ActiveFlipY,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconFlipY.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::ActiveRotateCCW,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconRotateCCW.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::ActiveRotateCW,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconRotateCW.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::MoveUp,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconMoveUp.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::MoveDown,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconMoveDown.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::MoveRight,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconMoveRight.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::MoveLeft,
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, ACTION_TIP_ICON_ALPHA),
                base_color_texture: Some(asset_server.load("ActionTipIconMoveLeft.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        );
        action_tip_materials.insert(
            PuzzleAction::NoAction,
            materials.add(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        );

        // Spawn the puzzle main entity, with solution and tiles children
        commands
            .entity(puzzle_entity)
            .insert(SpatialBundle::default())
            .insert(Name::new("Puzzle"))
            .insert(PuzzleAssets {
                tile_scale,
                solved_tile_scale,
                outline_color_active: Color::WHITE,
                outline_color_misplaced_misoriented: Color::Srgba(
                    bevy::color::palettes::css::PURPLE,
                ),
                outline_color_misplaced: Color::Srgba(bevy::color::palettes::css::RED),
                outline_color_misoriented: Color::Srgba(bevy::color::palettes::css::ORANGE),
                action_tip_materials,
            })
            .add_child(puzzle_solution)
            .add_child(puzzle_tiles)
            .add_child(puzzle_action_tip);
    }
}
