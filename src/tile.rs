use crate::prelude::*;

use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Default, Clone, Copy, Debug)]
pub enum CwRotation {
    #[default]
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
    pub fn rotate_cw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R90,
            R90 => R180,
            R180 => R370,
            R370 => R0,
        }
    }
    pub fn rotate_ccw(&self) -> Self {
        use CwRotation::*;
        match self {
            R0 => R370,
            R90 => R0,
            R180 => R90,
            R370 => R180,
        }
    }
}
#[derive(Debug)]
pub struct Tile {
    // When spawned, Bevy entity associated to this tile
    pub entity: Option<Entity>,
    // Defines if this tile image is flipped on the X axis compared to its initial state
    flipped_x: bool,
    // Defines if this tile image is flipped on the Y axis compared to its initial state
    flipped_y: bool,
    // Defines the tile image clock-wise rotation compated to its initial state.
    rotation: CwRotation,
    // Defines the tile position within the original image.
    // This will always be (0,0) for a puzzle with type FromSeparateImage
    pub position: (usize, usize),
}
impl Tile {
    pub fn new(position: (usize, usize)) -> Tile {
        Tile {
            entity: None,
            flipped_x: false,
            flipped_y: false,
            rotation: CwRotation::R0,
            position,
        }
    }
    pub fn compute_rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Z, self.rotation.angle())
    }
    pub fn compute_mesh(&self, size: (usize, usize)) -> Mesh {
        let incr_x = 1.0 / (size.1 as f32);
        let incr_y = 1.0 / (size.0 as f32);
        let mut uv_x1 = self.position.1 as f32 * incr_x;
        let mut uv_x2 = (self.position.1 + 1) as f32 * incr_x;
        let mut uv_y1 = (size.0 - 1 - self.position.0) as f32 * incr_y;
        let mut uv_y2 = (size.0 - self.position.0) as f32 * incr_y;
        if self.flipped_x {
            (uv_x1, uv_x2) = (uv_x2, uv_x1);
        }
        if self.flipped_y {
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
        mesh
    }
    pub fn flip_x(&mut self) {
        self.flipped_x = !self.flipped_x;
    }
    pub fn flip_y(&mut self) {
        self.flipped_y = !self.flipped_y;
    }
    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.rotate_cw();
    }
    pub fn rotate_ccw(&mut self) {
        self.rotation = self.rotation.rotate_ccw();
    }
}
