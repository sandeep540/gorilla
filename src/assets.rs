use macroquad::prelude::*;

pub const VIRTUAL_WIDTH: f32 = 320.0;
pub const VIRTUAL_HEIGHT: f32 = 180.0;
pub const BUILDING_WIDTH: f32 = 24.0;
pub const GORILLA_RADIUS: f32 = 6.0;
pub const BANANA_RADIUS: f32 = 2.0;
pub const EXPLOSION_DURATION: f32 = 0.7;
pub const ROUND_BANNER_TIME: f32 = 1.4;
pub const MATCH_OVER_DELAY: f32 = 0.9;
pub const TRAJECTORY_STEPS: usize = 22;
pub const TRAJECTORY_DT: f32 = 0.08;

pub fn sky_top() -> Color {
    Color::from_rgba(249, 147, 76, 255)
}

pub fn sky_bottom() -> Color {
    Color::from_rgba(35, 48, 94, 255)
}

pub fn horizon_glow() -> Color {
    Color::from_rgba(250, 201, 120, 255)
}

pub fn building_color() -> Color {
    Color::from_rgba(38, 38, 58, 255)
}

pub fn building_highlight() -> Color {
    Color::from_rgba(59, 64, 96, 255)
}

pub fn window_lit() -> Color {
    Color::from_rgba(255, 209, 102, 255)
}

pub fn window_dim() -> Color {
    Color::from_rgba(80, 87, 120, 255)
}

pub fn player_one() -> Color {
    Color::from_rgba(133, 226, 255, 255)
}

pub fn player_two() -> Color {
    Color::from_rgba(255, 123, 123, 255)
}

pub fn banana() -> Color {
    Color::from_rgba(255, 230, 102, 255)
}

pub fn ui_panel() -> Color {
    Color::from_rgba(15, 19, 34, 230)
}

pub fn ui_outline() -> Color {
    Color::from_rgba(255, 239, 191, 255)
}

pub fn text_primary() -> Color {
    Color::from_rgba(255, 247, 224, 255)
}

pub fn text_muted() -> Color {
    Color::from_rgba(230, 225, 208, 255)
}

pub fn explosion() -> Color {
    Color::from_rgba(255, 167, 38, 255)
}

pub fn wind_arrow() -> Color {
    Color::from_rgba(164, 236, 255, 255)
}

pub fn sun() -> Color {
    Color::from_rgba(255, 240, 179, 255)
}
