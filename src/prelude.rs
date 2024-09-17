pub use crate::game_state::*;
pub use crate::puzzle::*;
pub use crate::tile::*;
pub use crate::ui::*;
pub use bevy::prelude::*;
pub use bevy_camera_blur::*;
pub use bevy_mod_outline::*;
pub use bevy_mod_picking::prelude::*;
pub use bevy_tweening::*;

pub const TILE_OCCUPANCY: f32 = 0.93;
pub const Z_PUZZLE_TILE: f32 = 0.;
pub const Z_PUZZLE_SOLUTION: f32 = 2.;
pub const Z_PUZZLE_ACTION_TIP: f32 = 1.;
pub const ACTION_TIP_GRID_ALPHA: f32 = 0.2;
pub const ACTION_TIP_ICON_ALPHA: f32 = 0.3;

pub const UI_HEADER_PX: f32 = 24.0;
pub const UI_TEXT_COLOR: Color = Color::Srgba(bevy::color::palettes::css::ANTIQUE_WHITE);
pub const UI_COLOR_1: Color = Color::srgb(103. / 255., 91. / 255., 153. / 255.);
pub const UI_COLOR_2: Color = Color::srgb(78. / 255., 68. / 255., 122. / 255.);
pub const UI_COLOR_3: Color = Color::srgb(44. / 255., 39. / 255., 69. / 255.);
pub const MAIN_BACKGROUND_COLOR: Color = Color::srgb(39. / 255., 34. / 255., 61. / 255.);

pub const BLUR_ANIMATION_DURATION: u64 = 300;
pub const ACTION_ANIMATION_DURATION: u64 = 150;

pub const BLUR: GaussianBlurSettings = GaussianBlurSettings {
    kernel_size: 31,
    sampling_distance_factor: 1.0,
};
