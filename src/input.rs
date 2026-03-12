use crate::assets;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct UiButton {
    pub rect: Rect,
    pub label: &'static str,
}

impl UiButton {
    pub fn hovered(self, mouse: Vec2) -> bool {
        self.rect.contains(mouse)
    }
}

pub fn screen_to_virtual(mouse: Vec2, viewport: Rect) -> Option<Vec2> {
    if !viewport.contains(mouse) {
        return None;
    }
    let x = (mouse.x - viewport.x) * assets::VIRTUAL_WIDTH / viewport.w;
    let y = (mouse.y - viewport.y) * assets::VIRTUAL_HEIGHT / viewport.h;
    Some(vec2(x, y))
}

pub fn button_clicked(button: UiButton, mouse: Vec2) -> bool {
    is_mouse_button_pressed(MouseButton::Left) && button.hovered(mouse)
}
