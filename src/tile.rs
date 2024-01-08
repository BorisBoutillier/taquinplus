use crate::prelude::*;

use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CwRotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}
impl CwRotation {
    pub fn angle(&self) -> f32 {
        use CwRotation::*;
        match self {
            R0 => 0.,
            R90 => -FRAC_PI_2,
            R180 => PI,
            R270 => FRAC_PI_2,
        }
    }
    pub fn rotate_cw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R90,
            R90 => R180,
            R180 => R270,
            R270 => R0,
        }
    }
    pub fn rotate_ccw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R270,
            R90 => R0,
            R180 => R90,
            R270 => R180,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Tile {
    // When spawned, Bevy entity associated to this tile
    pub entity: Option<Entity>,
    // Defines if this tile image is flipped on the X axis compared to its initial state
    flipped_x: bool,
    // Defines if this tile image is flipped on the Y axis compared to its initial state
    flipped_y: bool,
    // Defines the tile image clock-wise rotation compated to its initial state.
    pub rotation: CwRotation,
    // Defines the tile position within the original image.
    // This will always be (0,0) for a puzzle with type FromSeparateImage
    pub position: Coord,
    // Defines the full size of the puzzle this tile is in.
    pub puzzle_size: Coord,
}
impl Tile {
    pub fn new(position: Coord, puzzle_size: Coord) -> Tile {
        Tile {
            entity: None,
            flipped_x: false,
            flipped_y: false,
            rotation: CwRotation::R0,
            position,
            puzzle_size,
        }
    }
    pub fn compute_rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Z, self.rotation.angle())
    }
    pub fn compute_mesh(&self) -> Mesh {
        compute_tile_mesh(
            self.puzzle_size,
            self.position,
            self.flipped_x,
            self.flipped_y,
        )
    }
    pub fn flip_x(&mut self) {
        self.flipped_x = !self.flipped_x;
    }
    pub fn flip_y(&mut self) {
        self.flipped_y = !self.flipped_y;
    }
    pub fn is_flipped(&self) -> bool {
        !self.is_correctly_oriented() && (self.flipped_x | self.flipped_y)
    }
    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.rotate_cw();
    }
    pub fn rotate_ccw(&mut self) {
        self.rotation = self.rotation.rotate_ccw();
    }
    pub fn is_rotated(&self) -> bool {
        !self.is_correctly_oriented() && self.rotation != CwRotation::R0
    }
    // True if this tile is neither flipped nor rotated.
    // This takes care of the of case of Flip X and Y and Rotated 180, which is an invariant
    pub fn is_correctly_oriented(&self) -> bool {
        matches!(
            (self.flipped_x, self.flipped_y, self.rotation),
            (false, false, CwRotation::R0) | (true, true, CwRotation::R180)
        )
    }
}

pub fn compute_tile_mesh(size: Coord, position: Coord, flipped_x: bool, flipped_y: bool) -> Mesh {
    let mut mesh = Mesh::from(shape::Cube::new(1.));
    set_tile_mesh_uvs(&mut mesh, size, position, flipped_x, flipped_y);
    mesh
}
pub fn set_tile_mesh_uvs(
    mesh: &mut Mesh,
    size: Coord,
    position: Coord,
    flipped_x: bool,
    flipped_y: bool,
) {
    let incr_x = 1.0 / (size.1 as f32);
    let incr_y = 1.0 / (size.0 as f32);
    let mut uv_x1 = position.1 as f32 * incr_x;
    let mut uv_x2 = (position.1 + 1) as f32 * incr_x;
    let mut uv_y1 = (size.0 - 1 - position.0) as f32 * incr_y;
    let mut uv_y2 = (size.0 - position.0) as f32 * incr_y;
    if flipped_x {
        (uv_x1, uv_x2) = (uv_x2, uv_x1);
    }
    if flipped_y {
        (uv_y1, uv_y2) = (uv_y2, uv_y1);
    }
    #[rustfmt::skip]
    let uvs = vec![
        // Assigning the UV coords for the top side.
        [uv_x1, uv_y2], [uv_x2, uv_y2], [uv_x2, uv_y1], [uv_x1, uv_y1],
        // Other sides are uniform color of 0,0 pixel
        [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
        [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
        [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
        [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
        [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
}

pub fn set_tile_mesh_position(
    mesh: &mut Mesh,
    x_flipping_ratio: Option<f32>,
    y_flipping_ratio: Option<f32>,
) {
    let (min_x, max_x) = {
        let val = if let Some(ratio) = x_flipping_ratio {
            (ratio - 0.5).abs()
        } else {
            0.5
        };
        (-val, val)
    };
    let (min_y, max_y) = {
        let val = if let Some(ratio) = y_flipping_ratio {
            (ratio - 0.5).abs()
        } else {
            0.5
        };
        (-val, val)
    };
    let min_z = -0.5;
    let max_z = 0.5;
    let positions = vec![
        // Front
        [min_x, min_y, max_z],
        [max_x, min_y, max_z],
        [max_x, max_y, max_z],
        [min_x, max_y, max_z],
        // Back
        [min_x, max_y, min_z],
        [max_x, max_y, min_z],
        [max_x, min_y, min_z],
        [min_x, min_y, min_z],
        // Right
        [max_x, min_y, min_z],
        [max_x, max_y, min_z],
        [max_x, max_y, max_z],
        [max_x, min_y, max_z],
        // Left
        [min_x, min_y, max_z],
        [min_x, max_y, max_z],
        [min_x, max_y, min_z],
        [min_x, min_y, min_z],
        // Top
        [max_x, max_y, min_z],
        [min_x, max_y, min_z],
        [min_x, max_y, max_z],
        [max_x, max_y, max_z],
        // Bottom
        [max_x, min_y, max_z],
        [min_x, min_y, max_z],
        [min_x, min_y, min_z],
        [max_x, min_y, min_z],
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
}

pub struct MeshFlippingLens {
    pub tile: Tile,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flipped: bool,
}
impl MeshFlippingLens {
    pub fn new_flip_x(tile: Tile) -> Self {
        Self {
            flip_x: true,
            flip_y: false,
            flipped: false,
            tile,
        }
    }
    pub fn new_flip_y(tile: Tile) -> Self {
        Self {
            flip_x: false,
            flip_y: true,
            flipped: false,
            tile,
        }
    }
}

impl Lens<Mesh> for MeshFlippingLens {
    fn lerp(&mut self, target: &mut Mesh, ratio: f32) {
        if !self.flipped && ratio > 0.5 {
            self.flipped = true;
            set_tile_mesh_uvs(
                target,
                self.tile.puzzle_size,
                self.tile.position,
                self.tile.flipped_x,
                self.tile.flipped_y,
            );
        }
        set_tile_mesh_position(
            target,
            self.flip_x.then_some(ratio),
            self.flip_y.then_some(ratio),
        );
    }
}
