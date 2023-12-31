use std::f32::consts::{FRAC_PI_2, PI};

use crate::prelude::*;
pub enum CwRotation {
    R0,
    R90,
    R180,
    R370,
}
impl CwRotation {
    pub fn angle(&self) -> f32 {
        use CwRotation::*;
        match self {
            R0 => 0.,
            R90 => -FRAC_PI_2,
            R180 => PI,
            R370 => FRAC_PI_2,
        }
    }
    pub fn rot_cw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R90,
            R90 => R180,
            R180 => R370,
            R370 => R0,
        }
    }
    pub fn rot_ccw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R370,
            R90 => R0,
            R180 => R90,
            R370 => R180,
        }
    }
}

#[derive(Component)]
// Defines the position of this piece within the reference image
// Also contains the total number of pieces of the reference image
// for ease of use
pub struct Piece {
    pub x: isize,
    pub y: isize,
    pub len_x: isize,
    pub len_y: isize,
}

#[derive(Component)]
// Defines the flipped state of this piece in the puzzle.
pub struct Flipped {
    pub flipped_x: bool,
    pub flipped_y: bool,
}
#[derive(Component)]
pub struct Rotated(CwRotation);
impl Rotated {
    pub fn rot_cw(&mut self) {
        self.0 = self.0.rot_cw();
    }
    pub fn rot_ccw(&mut self) {
        self.0 = self.0.rot_ccw();
    }
}
impl Default for Rotated {
    fn default() -> Self {
        Self(CwRotation::R0)
    }
}

#[derive(Component)]
pub struct Position {
    pub x: isize,
    pub y: isize,
    pub len_x: isize,
    pub len_y: isize,
}

pub fn update_tile_flip(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    flips: Query<(Entity, &Piece, &Flipped), Changed<Flipped>>,
) {
    for (entity, piece, flipped) in flips.iter() {
        let incr_x = 1.0 / (piece.len_x as f32);
        let incr_y = 1.0 / (piece.len_y as f32);
        let mut uv_x1 = piece.x as f32 * incr_x;
        let mut uv_x2 = (piece.x + 1) as f32 * incr_x;
        let mut uv_y1 = (piece.len_y - 1 - piece.y) as f32 * incr_y;
        let mut uv_y2 = (piece.len_y - piece.y) as f32 * incr_y;
        if flipped.flipped_x {
            (uv_x1, uv_x2) = (uv_x2, uv_x1);
        }
        if flipped.flipped_y {
            (uv_y1, uv_y2) = (uv_y2, uv_y1);
        }
        let mut mesh = Mesh::from(shape::Cube::new(1.));
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
        commands.entity(entity).insert(meshes.add(mesh));
    }
}
pub fn update_tile_rotation(mut rotations: Query<(&mut Transform, &Rotated), Changed<Rotated>>) {
    for (mut transform, rotated) in rotations.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::Z, rotated.0.angle());
    }
}
pub fn update_tile_position(mut positions: Query<(&mut Transform, &Position), Changed<Position>>) {
    for (mut transform, position) in positions.iter_mut() {
        transform.translation.x =
            (position.x - (position.len_x / 2)) as f32 / position.len_x as f32;
        transform.translation.y =
            (position.y - (position.len_y / 2)) as f32 / position.len_y as f32;
    }
}
