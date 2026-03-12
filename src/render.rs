use crate::assets;
use crate::input::UiButton;
use crate::model::{hand_position, MatchState, PlayerId, RoundState, Settings, ThrowField};
use crate::scene::Scene;
use macroquad::prelude::*;

const UI_FONT_SCALE: f32 = 1.0;

pub async fn load_ui_font() -> Option<Font> {
    let candidates = [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    for path in candidates {
        if let Ok(font) = load_ttf_font(path).await {
            return Some(font);
        }
    }
    None
}

pub fn viewport_rect() -> Rect {
    let scale = (screen_width() / assets::VIRTUAL_WIDTH)
        .floor()
        .min((screen_height() / assets::VIRTUAL_HEIGHT).floor())
        .max(1.0);
    let width = assets::VIRTUAL_WIDTH * scale;
    let height = assets::VIRTUAL_HEIGHT * scale;
    Rect::new(
        (screen_width() - width) * 0.5,
        (screen_height() - height) * 0.5,
        width,
        height,
    )
}

pub fn begin_virtual_camera(viewport: Rect, shake: Vec2) -> RenderTarget {
    let target_width = viewport.w.max(1.0) as u32;
    let target_height = viewport.h.max(1.0) as u32;
    let target = render_target(target_width, target_height);
    target.texture.set_filter(FilterMode::Nearest);
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        zoom: vec2(2.0 / assets::VIRTUAL_WIDTH, -2.0 / assets::VIRTUAL_HEIGHT),
        target: vec2(
            assets::VIRTUAL_WIDTH * 0.5 - shake.x,
            assets::VIRTUAL_HEIGHT * 0.5 - shake.y,
        ),
        ..Default::default()
    });
    target
}

pub fn draw_virtual_to_screen(target: RenderTarget, viewport: Rect) {
    set_default_camera();
    clear_background(BLACK);
    draw_vertical_gradient(
        Rect::new(0.0, 0.0, screen_width(), screen_height()),
        Color::from_rgba(11, 11, 14, 255),
        Color::from_rgba(26, 26, 32, 255),
    );
    draw_texture_ex(
        &target.texture,
        viewport.x,
        viewport.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(viewport.w, viewport.h)),
            flip_y: true,
            ..Default::default()
        },
    );
}

pub fn draw_gameplay(
    match_state: &MatchState,
    preview: &[Vec2],
) {
    draw_background();
    draw_city(&match_state.round);
    draw_sun(match_state.round.wind.strength);
    draw_gorillas(&match_state.round);
    draw_projectile(&match_state.round);
    draw_explosion(&match_state.round);
    draw_preview(preview);
}

fn draw_background() {
    draw_vertical_gradient(
        Rect::new(0.0, 0.0, assets::VIRTUAL_WIDTH, assets::VIRTUAL_HEIGHT),
        assets::sky_top(),
        assets::sky_bottom(),
    );
    draw_circle(
        assets::VIRTUAL_WIDTH * 0.5,
        assets::VIRTUAL_HEIGHT * 0.88,
        55.0,
        assets::horizon_glow(),
    );
}

fn draw_vertical_gradient(rect: Rect, top: Color, bottom: Color) {
    let steps = rect.h.max(1.0) as i32;
    for index in 0..steps {
        let t = index as f32 / steps as f32;
        let color = Color::new(
            top.r + (bottom.r - top.r) * t,
            top.g + (bottom.g - top.g) * t,
            top.b + (bottom.b - top.b) * t,
            top.a + (bottom.a - top.a) * t,
        );
        draw_rectangle(rect.x, rect.y + index as f32, rect.w, 1.5, color);
    }
}

fn draw_city(round: &RoundState) {
    for building in &round.city.buildings {
        draw_rectangle(
            building.rect.x,
            building.rect.y,
            building.rect.w,
            building.rect.h,
            assets::building_color(),
        );
        draw_line(
            building.rect.x,
            building.rect.y,
            building.rect.x + building.rect.w,
            building.rect.y,
            1.0,
            assets::building_highlight(),
        );
        for (window, lit) in &building.windows {
            draw_rectangle(
                window.x,
                window.y,
                window.w,
                window.h,
                if *lit {
                    assets::window_lit()
                } else {
                    assets::window_dim()
                },
            );
        }
    }
}

fn draw_sun(wind: f32) {
    let center = vec2(assets::VIRTUAL_WIDTH * 0.5, 24.0);
    draw_circle(center.x, center.y, 12.0, assets::sun());
    draw_text(":)", center.x - 7.0, center.y + 3.0, 14.0, BLACK);
    let arrow_len = wind.abs() * 1.8;
    draw_line(center.x - 20.0, center.y + 20.0, center.x - 20.0 + arrow_len.signum() * arrow_len.max(8.0), center.y + 20.0, 2.0, assets::wind_arrow());
    draw_triangle(
        vec2(center.x - 20.0 + arrow_len.signum() * arrow_len.max(8.0), center.y + 20.0),
        vec2(center.x - 24.0 + arrow_len.signum() * arrow_len.max(8.0), center.y + 18.0),
        vec2(center.x - 24.0 + arrow_len.signum() * arrow_len.max(8.0), center.y + 22.0),
        assets::wind_arrow(),
    );
}

pub fn draw_hud_overlay(viewport: Rect, match_state: &MatchState, settings: &Settings, scene: Scene, font: Option<&Font>) {
    let panel = Rect::new(8.0, 8.0, assets::VIRTUAL_WIDTH - 16.0, 44.0);
    draw_panel(viewport, panel, Color::from_rgba(10, 14, 24, 235), 1.0);
    draw_ui_text(
        viewport,
        font,
        &format!("{}: {}", match_state.players[0].name, match_state.players[0].score),
        panel.x + 6.0,
        panel.y + 15.0,
        15,
        assets::text_primary(),
    );
    let right = format!("{}: {}", match_state.players[1].name, match_state.players[1].score);
    let right_width = measure_ui_text(&right, font, viewport, 15).width;
    draw_ui_text(
        viewport,
        font,
        &right,
        panel.x + panel.w - right_width - 6.0,
        panel.y + 15.0,
        15,
        assets::text_primary(),
    );
    let round_info = format!("Round {}  Goal {}  Wind {:+.0}", match_state.round_number, settings.target_score, match_state.round.wind.strength);
    draw_ui_text(viewport, font, &round_info, panel.x + 6.0, panel.y + 33.0, 14, assets::text_muted());
    if scene == Scene::RoundPlaying && match_state.round.projectile.is_none() {
        let aiming = format!("{} aiming", match_state.round.active_player.label());
        let aiming_width = measure_ui_text(&aiming, font, viewport, 14).width;
        draw_ui_text(
            viewport,
            font,
            &aiming,
            panel.x + panel.w - aiming_width - 6.0,
            panel.y + 33.0,
            14,
            assets::text_primary(),
        );
    }
}

pub fn draw_throw_input_overlay(viewport: Rect, round: &RoundState, scene: Scene, font: Option<&Font>) {
    if scene != Scene::RoundPlaying || round.projectile.is_some() {
        return;
    }
    let panel = Rect::new(8.0, assets::VIRTUAL_HEIGHT - 52.0, assets::VIRTUAL_WIDTH - 16.0, 44.0);
    draw_panel(viewport, panel, Color::from_rgba(9, 14, 25, 242), 1.5);

    draw_ui_text(viewport, font, "Throw Input", panel.x + 8.0, panel.y + 14.0, 14, assets::text_primary());
    draw_ui_text(
        viewport,
        font,
        &format!("{} turn", round.active_player.label()),
        panel.x + panel.w - 98.0,
        panel.y + 14.0,
        14,
        assets::text_primary(),
    );

    let field_w = (panel.w - 30.0) / 2.0;
    let angle_rect = Rect::new(panel.x + 8.0, panel.y + 20.0, field_w, 16.0);
    let velocity_rect = Rect::new(panel.x + 16.0 + field_w, panel.y + 20.0, field_w, 16.0);
    draw_input_box(
        viewport,
        angle_rect,
        &format!("Angle (0-90): {}", round.throw_input.angle_text),
        round.throw_input.focus == ThrowField::Angle,
        font,
    );
    draw_input_box(
        viewport,
        velocity_rect,
        &format!("Velocity (10-140): {}", round.throw_input.velocity_text),
        round.throw_input.focus == ThrowField::Velocity,
        font,
    );
}

fn draw_input_box(viewport: Rect, rect: Rect, label: &str, focused: bool, font: Option<&Font>) {
    let border = if focused {
        assets::player_one()
    } else {
        assets::ui_outline()
    };
    draw_panel(viewport, rect, Color::from_rgba(255, 255, 255, 8), 1.0);
    draw_rect_outline(viewport, rect, 1.0, border);
    draw_ui_text(viewport, font, label, rect.x + 4.0, rect.y + 11.5, 12, assets::text_primary());
}

fn draw_gorillas(round: &RoundState) {
    for gorilla in &round.gorillas {
        let color = match gorilla.player {
            PlayerId::One => assets::player_one(),
            PlayerId::Two => assets::player_two(),
        };
        draw_circle(gorilla.position.x, gorilla.position.y, assets::GORILLA_RADIUS, color);
        let bob = if gorilla.celebrating > 0.0 { (get_time() as f32 * 18.0).sin() * 1.5 } else { 0.0 };
        let arm_end = hand_position(gorilla) + vec2(0.0, bob);
        draw_line(gorilla.position.x, gorilla.position.y - 2.0, arm_end.x, arm_end.y, 2.0, color);
        draw_circle(gorilla.position.x - 2.0, gorilla.position.y - 1.5, 1.0, BLACK);
        draw_circle(gorilla.position.x + 2.0, gorilla.position.y - 1.5, 1.0, BLACK);
    }
}

fn draw_projectile(round: &RoundState) {
    if let Some(projectile) = &round.projectile {
        for (index, point) in projectile.trail.iter().enumerate() {
            let alpha = (index as f32 + 1.0) / projectile.trail.len().max(1) as f32;
            draw_circle(point.x, point.y, 1.0, Color::new(1.0, 0.9, 0.5, alpha * 0.5));
        }
        draw_circle(
            projectile.position.x,
            projectile.position.y,
            assets::BANANA_RADIUS,
            assets::banana(),
        );
    }
}

fn draw_explosion(round: &RoundState) {
    if let Some(explosion) = &round.explosion {
        let t = 1.0 - (explosion.timer / assets::EXPLOSION_DURATION).clamp(0.0, 1.0);
        let radius = explosion.max_radius * t;
        draw_circle(explosion.position.x, explosion.position.y, radius, assets::explosion());
        draw_circle_lines(explosion.position.x, explosion.position.y, radius + 3.0, 2.0, YELLOW);
    }
}

fn draw_preview(preview: &[Vec2]) {
    for point in preview {
        draw_circle(point.x, point.y, 1.0, Color::from_rgba(255, 255, 255, 170));
    }
}

pub fn draw_banner_overlay(viewport: Rect, match_state: &MatchState, scene: Scene, font: Option<&Font>) {
    let round = &match_state.round;
    if round.banner_timer > 0.0 && scene == Scene::MatchIntro {
        centered_panel(
            viewport,
            Rect::new(80.0, 60.0, 160.0, 38.0),
            &format!("Round {}", match_state.round_number),
            &format!("{} starts", round.active_player.label()),
            font,
        );
    }
    if scene == Scene::MatchOver {
        let winner = if match_state.players[0].score >= match_state.target_score {
            &match_state.players[0].name
        } else {
            &match_state.players[1].name
        };
        centered_panel(
            viewport,
            Rect::new(62.0, 54.0, 196.0, 52.0),
            "Match Complete",
            &format!("{winner} wins the skyline showdown"),
            font,
        );
    }
}

pub fn menu_buttons() -> [UiButton; 3] {
    [
        UiButton {
            rect: Rect::new(102.0, 98.0, 116.0, 16.0),
            label: "Start Match",
        },
        UiButton {
            rect: Rect::new(102.0, 118.0, 116.0, 16.0),
            label: "Settings",
        },
        UiButton {
            rect: Rect::new(102.0, 138.0, 116.0, 16.0),
            label: "Quit",
        },
    ]
}

pub fn draw_menu_overlay(viewport: Rect, selected_settings: &Settings, font: Option<&Font>) -> [UiButton; 3] {
    let panel = Rect::new(56.0, 28.0, 208.0, 124.0);
    draw_panel(viewport, panel, assets::ui_panel(), 2.0);
    let title = "GORILLA";
    let subtitle = "Skyline duel for two players";
    let points = format!("First to {} points", selected_settings.target_score);
    let title_dims = measure_ui_text(title, font, viewport, 26);
    let subtitle_dims = measure_ui_text(subtitle, font, viewport, 11);
    let points_dims = measure_ui_text(&points, font, viewport, 10);
    draw_ui_text(viewport, font, title, panel.x + (panel.w - title_dims.width) * 0.5, 58.0, 26, assets::text_primary());
    draw_ui_text(viewport, font, subtitle, panel.x + (panel.w - subtitle_dims.width) * 0.5, 74.0, 11, assets::text_muted());
    draw_ui_text(
        viewport,
        font,
        &points,
        panel.x + (panel.w - points_dims.width) * 0.5,
        88.0,
        10,
        assets::text_muted(),
    );

    let buttons = menu_buttons();
    for button in buttons {
        draw_button(viewport, button, None, font);
    }
    buttons
}

pub fn settings_buttons() -> [UiButton; 7] {
    [
        UiButton { rect: Rect::new(64.0, 54.0, 192.0, 14.0), label: "Target Score" },
        UiButton { rect: Rect::new(64.0, 72.0, 192.0, 14.0), label: "Trajectory Preview" },
        UiButton { rect: Rect::new(64.0, 90.0, 192.0, 14.0), label: "Screen Shake" },
        UiButton { rect: Rect::new(64.0, 108.0, 192.0, 14.0), label: "Fullscreen" },
        UiButton { rect: Rect::new(64.0, 126.0, 92.0, 14.0), label: "Master Volume" },
        UiButton { rect: Rect::new(164.0, 126.0, 92.0, 14.0), label: "SFX Volume" },
        UiButton { rect: Rect::new(102.0, 146.0, 116.0, 14.0), label: "Back" },
    ]
}

pub fn draw_settings_overlay(viewport: Rect, settings: &Settings, font: Option<&Font>) -> [UiButton; 7] {
    centered_panel(
        viewport,
        Rect::new(44.0, 18.0, 232.0, 144.0),
        "Settings",
        "Click rows to change values",
        font,
    );
    let buttons = settings_buttons();
    let values = [
        format!("Target Score: {}", settings.target_score),
        format!("Trajectory Preview: {}", if settings.trajectory_preview { "On" } else { "Off" }),
        format!("Screen Shake: {}", if settings.screen_shake { "On" } else { "Off" }),
        format!("Fullscreen: {}", if settings.fullscreen { "On" } else { "Off" }),
        format!("Master: {:>3.0}%", settings.master_volume * 100.0),
        format!("SFX: {:>3.0}%", settings.sfx_volume * 100.0),
        "Back".to_string(),
    ];
    for (button, value) in buttons.into_iter().zip(values) {
        draw_button(viewport, button, Some(&value), font);
    }
    buttons
}

pub fn pause_buttons() -> [UiButton; 3] {
    [
        UiButton { rect: Rect::new(102.0, 94.0, 116.0, 14.0), label: "Resume" },
        UiButton { rect: Rect::new(102.0, 112.0, 116.0, 14.0), label: "Restart Match" },
        UiButton { rect: Rect::new(102.0, 130.0, 116.0, 14.0), label: "Main Menu" },
    ]
}

pub fn draw_pause_overlay(viewport: Rect, font: Option<&Font>) -> [UiButton; 3] {
    let full = rect_to_screen(viewport, Rect::new(0.0, 0.0, assets::VIRTUAL_WIDTH, assets::VIRTUAL_HEIGHT));
    draw_rectangle(full.x, full.y, full.w, full.h, Color::from_rgba(4, 7, 14, 140));
    centered_panel(
        viewport,
        Rect::new(72.0, 48.0, 176.0, 104.0),
        "Paused",
        "Resume, restart, or return to the main menu",
        font,
    );
    let buttons = pause_buttons();
    for button in buttons {
        draw_button(viewport, button, None, font);
    }
    buttons
}

pub fn draw_help_footer(viewport: Rect, scene: Scene, font: Option<&Font>) {
    if scene == Scene::MainMenu || scene == Scene::Settings {
        let copy = "Esc pauses gameplay. Cmd+Enter toggles fullscreen.";
        draw_ui_text(viewport, font, copy, 10.0, assets::VIRTUAL_HEIGHT - 8.0, 9, assets::text_primary());
    }
}

fn draw_button(viewport: Rect, button: UiButton, text_override: Option<&str>, font: Option<&Font>) {
    draw_panel(viewport, button.rect, Color::from_rgba(255, 255, 255, 12), 1.0);
    let text = text_override.unwrap_or(button.label);
    let text_size = 10;
    let text_dims = measure_ui_text(text, font, viewport, text_size);
    draw_ui_text(
        viewport,
        font,
        text,
        button.rect.x + (button.rect.w - text_dims.width) * 0.5,
        button.rect.y + 10.5,
        text_size,
        assets::text_primary(),
    );
}

fn centered_panel(viewport: Rect, rect: Rect, title: &str, subtitle: &str, font: Option<&Font>) {
    draw_panel(viewport, rect, assets::ui_panel(), 2.0);
    let title_dims = measure_ui_text(title, font, viewport, 18);
    let subtitle_dims = measure_ui_text(subtitle, font, viewport, 9);
    draw_ui_text(
        viewport,
        font,
        title,
        rect.x + (rect.w - title_dims.width) * 0.5,
        rect.y + 24.0,
        18,
        assets::text_primary(),
    );
    draw_ui_text(
        viewport,
        font,
        subtitle,
        rect.x + (rect.w - subtitle_dims.width) * 0.5,
        rect.y + 39.0,
        9,
        assets::text_muted(),
    );
}

fn draw_ui_text(viewport: Rect, font: Option<&Font>, text: &str, x: f32, y: f32, size: u16, color: Color) {
    let scale = viewport.w / assets::VIRTUAL_WIDTH;
    draw_text_ex(
        text,
        viewport.x + x * scale,
        viewport.y + y * scale,
        TextParams {
            font,
            font_size: ui_font_size(scale, size),
            color,
            ..Default::default()
        },
    );
}

fn measure_ui_text(text: &str, font: Option<&Font>, viewport: Rect, size: u16) -> TextDimensions {
    let scale = viewport.w / assets::VIRTUAL_WIDTH;
    let dimensions = measure_text(text, font, ui_font_size(scale, size), 1.0);
    TextDimensions {
        width: dimensions.width / scale,
        height: dimensions.height / scale,
        offset_y: dimensions.offset_y / scale,
    }
}

fn ui_font_size(scale: f32, base: u16) -> u16 {
    ((base as f32) * scale * UI_FONT_SCALE).round().clamp(8.0, 96.0) as u16
}

fn draw_panel(viewport: Rect, rect: Rect, fill: Color, thickness: f32) {
    let screen = rect_to_screen(viewport, rect);
    draw_rectangle(screen.x, screen.y, screen.w, screen.h, fill);
    draw_rectangle_lines(screen.x, screen.y, screen.w, screen.h, thickness * screen_scale(viewport), assets::ui_outline());
}

fn draw_rect_outline(viewport: Rect, rect: Rect, thickness: f32, color: Color) {
    let screen = rect_to_screen(viewport, rect);
    draw_rectangle_lines(screen.x, screen.y, screen.w, screen.h, thickness * screen_scale(viewport), color);
}

fn rect_to_screen(viewport: Rect, rect: Rect) -> Rect {
    let scale = screen_scale(viewport);
    Rect::new(
        viewport.x + rect.x * scale,
        viewport.y + rect.y * scale,
        rect.w * scale,
        rect.h * scale,
    )
}

fn screen_scale(viewport: Rect) -> f32 {
    viewport.w / assets::VIRTUAL_WIDTH
}
