pub use crate::game_state::*;
pub use crate::gaussian_blur::*;
pub use crate::puzzle::*;
pub use crate::tile::*;
pub use crate::ui::*;
pub use bevy::prelude::*;
pub use bevy_mod_outline::*;
pub use bevy_mod_picking::prelude::*;
pub use bevy_tweening::*;

pub const UI_HEADER_PX: f32 = 24.0;
pub const UI_TEXT_COLOR: Color = Color::ANTIQUE_WHITE;
pub const UI_COLOR_1: Color = Color::rgb(103. / 255., 91. / 255., 153. / 255.);
pub const UI_COLOR_2: Color = Color::rgb(78. / 255., 68. / 255., 122. / 255.);
pub const UI_COLOR_3: Color = Color::rgb(44. / 255., 39. / 255., 69. / 255.);
pub const MAIN_BACKGROUND_COLOR: Color = Color::rgb(39. / 255., 34. / 255., 61. / 255.);

pub const BLUR_ANIMATION_DURATION: u64 = 300;
pub const ACTION_ANIMATION_DURATION: u64 = 150;

pub const BLUR: GaussianBlurSettings = GaussianBlurSettings {
    sigma: 15.0,
    kernel_size: 60,
    sample_rate_factor: 1.0,
    _webgl2_padding: 0.,
};
pub const NO_BLUR: GaussianBlurSettings = GaussianBlurSettings {
    sigma: 0.0,
    kernel_size: 1,
    sample_rate_factor: 1.0,
    _webgl2_padding: 0.,
};
