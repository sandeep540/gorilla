use crate::assets;
use crate::audio::AudioState;
use crate::input::{button_clicked, screen_to_virtual};
use crate::model::{
    default_throw_input, hand_position, new_match, reset_scores, start_next_round, GameState, PlayerId,
    ProjectileState,
};
use crate::physics::{resolve_collision, step_projectile, trajectory_preview, winner_for_hit, Collision};
use crate::persistence::{load_settings, save_settings};
use crate::render;
use crate::scene::Scene;
use macroquad::prelude::*;

pub struct App {
    scene: Scene,
    state: GameState,
    audio: AudioState,
    ui_font: Option<Font>,
    paused: bool,
    quit_requested: bool,
    shake_timer: f32,
    intro_timer: f32,
}

impl App {
    pub async fn new() -> Self {
        let settings = load_settings();
        set_fullscreen(settings.fullscreen);
        let state = GameState {
            match_state: new_match(&settings),
            settings,
        };
        let audio = AudioState::new(&state.settings).await;
        let ui_font = render::load_ui_font().await;
        Self {
            scene: Scene::MainMenu,
            state,
            audio,
            ui_font,
            paused: false,
            quit_requested: false,
            shake_timer: 0.0,
            intro_timer: 0.0,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }

    pub fn update(&mut self) {
        let dt = get_frame_time().min(1.0 / 30.0);
        self.shake_timer = (self.shake_timer - dt * 2.6).max(0.0);
        self.handle_global_input();

        if self.paused {
            let viewport = render::viewport_rect();
            let mouse_virtual = screen_to_virtual(mouse_position().into(), viewport);
            self.update_pause_overlay(mouse_virtual);
            return;
        }

        match self.scene {
            Scene::MainMenu => self.update_menu(),
            Scene::Settings => self.update_settings(),
            Scene::MatchIntro => self.update_intro(dt),
            Scene::RoundPlaying => self.update_round_playing(dt),
            Scene::RoundResolve => self.update_round_resolve(dt),
            Scene::MatchOver => self.update_match_over(dt),
        }
    }

    pub fn draw(&self) {
        let viewport = render::viewport_rect();
        let shake = self.current_shake();
        let target = render::begin_virtual_camera(viewport, shake);

        let preview = if self.scene == Scene::RoundPlaying
            && self.state.settings.trajectory_preview
            && self.state.match_state.round.projectile.is_none()
        {
            let active_player = self.state.match_state.round.active_player;
            let player = active_player.index();
            let start = hand_position(&self.state.match_state.round.gorillas[player]);
            if let Some(velocity) = parse_throw_velocity(
                active_player,
                &self.state.match_state.round.throw_input.angle_text,
                &self.state.match_state.round.throw_input.velocity_text,
            ) {
                trajectory_preview(&self.state.match_state.round, start, velocity)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        render::draw_gameplay(
            &self.state.match_state,
            &preview,
        );
        render::draw_virtual_to_screen(target, viewport);
        render::draw_hud_overlay(viewport, &self.state.match_state, &self.state.settings, self.scene, self.ui_font.as_ref());
        render::draw_banner_overlay(viewport, &self.state.match_state, self.scene, self.ui_font.as_ref());
        if !self.paused {
            render::draw_throw_input_overlay(viewport, &self.state.match_state.round, self.scene, self.ui_font.as_ref());
            render::draw_help_footer(viewport, self.scene, self.ui_font.as_ref());
        }

        match self.scene {
            Scene::MainMenu => {
                render::draw_menu_overlay(viewport, &self.state.settings, self.ui_font.as_ref());
            }
            Scene::Settings => {
                render::draw_settings_overlay(viewport, &self.state.settings, self.ui_font.as_ref());
            }
            _ => {}
        }

        if self.paused {
            render::draw_pause_overlay(viewport, self.ui_font.as_ref());
        }
    }

    fn handle_global_input(&mut self) {
        if is_key_pressed(KeyCode::Enter) && (is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper)) {
            self.state.settings.fullscreen = !self.state.settings.fullscreen;
            set_fullscreen(self.state.settings.fullscreen);
            save_settings(&self.state.settings);
            self.audio.ui();
        }

        if is_key_pressed(KeyCode::Escape) {
            match self.scene {
                Scene::RoundPlaying | Scene::RoundResolve => {
                    self.paused = !self.paused;
                    self.audio.ui();
                }
                Scene::Settings => {
                    self.scene = Scene::MainMenu;
                    save_settings(&self.state.settings);
                    self.audio.ui();
                }
                _ => {}
            }
        }
    }

    fn update_menu(&mut self) {
        let viewport = render::viewport_rect();
        let Some(mouse) = screen_to_virtual(mouse_position().into(), viewport) else {
            return;
        };
        let buttons = render::menu_buttons();
        if button_clicked(buttons[0], mouse) {
            self.begin_new_match();
        } else if button_clicked(buttons[1], mouse) {
            self.scene = Scene::Settings;
            self.audio.ui();
        } else if button_clicked(buttons[2], mouse) {
            self.quit_requested = true;
        }
    }

    fn update_settings(&mut self) {
        let viewport = render::viewport_rect();
        let Some(mouse) = screen_to_virtual(mouse_position().into(), viewport) else {
            return;
        };
        let buttons = render::settings_buttons();
        if button_clicked(buttons[0], mouse) {
            self.state.settings.target_score = match self.state.settings.target_score {
                3 => 5,
                5 => 7,
                _ => 3,
            };
        } else if button_clicked(buttons[1], mouse) {
            self.state.settings.trajectory_preview = !self.state.settings.trajectory_preview;
        } else if button_clicked(buttons[2], mouse) {
            self.state.settings.screen_shake = !self.state.settings.screen_shake;
        } else if button_clicked(buttons[3], mouse) {
            self.state.settings.fullscreen = !self.state.settings.fullscreen;
            set_fullscreen(self.state.settings.fullscreen);
        } else if button_clicked(buttons[4], mouse) {
            self.state.settings.master_volume = cycle_volume(self.state.settings.master_volume);
        } else if button_clicked(buttons[5], mouse) {
            self.state.settings.sfx_volume = cycle_volume(self.state.settings.sfx_volume);
        } else if button_clicked(buttons[6], mouse) {
            self.scene = Scene::MainMenu;
        } else {
            return;
        }
        self.audio.sync(&self.state.settings);
        save_settings(&self.state.settings);
        self.audio.ui();
    }

    fn update_intro(&mut self, dt: f32) {
        self.intro_timer -= dt;
        self.state.match_state.round.banner_timer = self.intro_timer;
        if self.intro_timer <= 0.0 {
            self.scene = Scene::RoundPlaying;
        }
    }

    fn update_round_playing(&mut self, dt: f32) {
        let viewport = render::viewport_rect();
        let mouse_virtual = screen_to_virtual(mouse_position().into(), viewport);
        self.update_pause_overlay(mouse_virtual);

        {
            let round = &mut self.state.match_state.round;
            if let Some(projectile) = &mut round.projectile {
                step_projectile(projectile, round.wind, round.gravity, dt);
                match resolve_collision(projectile.position, &round.city.buildings, &round.gorillas) {
                    Collision::None => {}
                    Collision::Building | Collision::OutOfBounds => {
                        projectile.alive = false;
                        round.explosion = Some(crate::model::Explosion {
                            position: projectile.position,
                            timer: assets::EXPLOSION_DURATION,
                            max_radius: 12.0,
                        });
                        round.projectile = None;
                        round.active_player = round.active_player.other();
                        round.throw_input = default_throw_input(round.active_player);
                        self.scene = Scene::RoundResolve;
                        self.shake_timer = 0.22;
                        self.audio.impact();
                    }
                    Collision::Gorilla(hit_index) => {
                        projectile.alive = false;
                        round.explosion = Some(crate::model::Explosion {
                            position: projectile.position,
                            timer: assets::EXPLOSION_DURATION,
                            max_radius: 20.0,
                        });
                        let winner_index = winner_for_hit(hit_index);
                        let winner_id = if winner_index == 0 { PlayerId::One } else { PlayerId::Two };
                        round.winner = Some(winner_id);
                        round.gorillas[winner_index].celebrating = 1.2;
                        self.scene = Scene::RoundResolve;
                        self.shake_timer = 0.35;
                        self.audio.win();
                    }
                }
                return;
            }
        }

        self.capture_throw_text_input();
        let round = &mut self.state.match_state.round;
        let active_index = round.active_player.index();
        if let Some(velocity) = parse_throw_velocity(
            round.active_player,
            &round.throw_input.angle_text,
            &round.throw_input.velocity_text,
        ) {
            round.gorillas[active_index].arm_angle = velocity.y.atan2(velocity.x);
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                let start = hand_position(&round.gorillas[active_index]);
                round.projectile = Some(ProjectileState {
                    position: start,
                    velocity,
                    trail: Vec::new(),
                    alive: true,
                });
                self.audio.throw();
            }
        }
    }

    fn update_round_resolve(&mut self, dt: f32) {
        self.update_pause_overlay(screen_to_virtual(mouse_position().into(), render::viewport_rect()));
        let round = &mut self.state.match_state.round;
        if let Some(explosion) = &mut round.explosion {
            explosion.timer -= dt;
            if explosion.timer <= 0.0 {
                round.explosion = None;
            }
        }
        for gorilla in &mut round.gorillas {
            gorilla.celebrating = (gorilla.celebrating - dt).max(0.0);
        }
        if round.explosion.is_some() {
            return;
        }
        if let Some(winner) = round.winner.take() {
            let index = winner.index();
            self.state.match_state.players[index].score += 1;
            if self.state.match_state.players[index].score >= self.state.match_state.target_score {
                self.intro_timer = assets::MATCH_OVER_DELAY;
                self.scene = Scene::MatchOver;
            } else {
                start_next_round(&mut self.state.match_state);
                self.scene = Scene::MatchIntro;
                self.intro_timer = assets::ROUND_BANNER_TIME;
            }
        } else {
            self.scene = Scene::RoundPlaying;
        }
    }

    fn update_match_over(&mut self, dt: f32) {
        self.intro_timer -= dt;
        if self.intro_timer > 0.0 {
            return;
        }
        let viewport = render::viewport_rect();
        let Some(mouse) = screen_to_virtual(mouse_position().into(), viewport) else {
            return;
        };
        let buttons = render::pause_buttons();
        if button_clicked(buttons[0], mouse) || button_clicked(buttons[1], mouse) {
            self.begin_new_match();
        } else if button_clicked(buttons[2], mouse) {
            self.scene = Scene::MainMenu;
            self.audio.ui();
        }
    }

    fn update_pause_overlay(&mut self, mouse_virtual: Option<Vec2>) {
        if !self.paused {
            return;
        }
        let Some(mouse) = mouse_virtual else {
            return;
        };
        let buttons = render::pause_buttons();
        if button_clicked(buttons[0], mouse) {
            self.paused = false;
        } else if button_clicked(buttons[1], mouse) {
            self.begin_new_match();
            self.paused = false;
        } else if button_clicked(buttons[2], mouse) {
            self.paused = false;
            self.scene = Scene::MainMenu;
        } else {
            return;
        }
        self.audio.ui();
    }

    fn begin_new_match(&mut self) {
        let settings = self.state.settings.clone();
        let mut match_state = new_match(&settings);
        reset_scores(&mut match_state, &settings);
        self.state.match_state = match_state;
        self.scene = Scene::MatchIntro;
        self.intro_timer = assets::ROUND_BANNER_TIME;
        self.paused = false;
        save_settings(&self.state.settings);
        self.audio.sync(&self.state.settings);
        self.audio.ui();
    }

    fn current_shake(&self) -> Vec2 {
        if !self.state.settings.screen_shake || self.shake_timer <= 0.0 {
            return Vec2::ZERO;
        }
        let t = get_time() as f32 * 70.0;
        let intensity = self.shake_timer * 2.2;
        vec2(t.sin() * intensity, (t * 1.3).cos() * intensity)
    }
}

fn cycle_volume(current: f32) -> f32 {
    match (current * 10.0).round() as i32 {
        10 => 0.0,
        value => ((value + 2) as f32 / 10.0).min(1.0),
    }
}

fn parse_throw_velocity(player: PlayerId, angle_text: &str, velocity_text: &str) -> Option<Vec2> {
    let angle = angle_text.parse::<f32>().ok()?;
    let velocity = velocity_text.parse::<f32>().ok()?;
    if !(0.0..=90.0).contains(&angle) {
        return None;
    }
    if !(10.0..=140.0).contains(&velocity) {
        return None;
    }
    let mirrored_angle = match player {
        PlayerId::One => angle,
        PlayerId::Two => 180.0 - angle,
    };
    let radians = mirrored_angle.to_radians();
    Some(vec2(radians.cos() * velocity, -radians.sin() * velocity))
}

impl App {
    fn capture_throw_text_input(&mut self) {
        let throw_input = &mut self.state.match_state.round.throw_input;
        if is_key_pressed(KeyCode::Tab) {
            throw_input.focus = match throw_input.focus {
                crate::model::ThrowField::Angle => crate::model::ThrowField::Velocity,
                crate::model::ThrowField::Velocity => crate::model::ThrowField::Angle,
            };
        }

        while let Some(character) = get_char_pressed() {
            if character.is_ascii_digit() || character == '.' {
                match throw_input.focus {
                    crate::model::ThrowField::Angle => {
                        if throw_input.angle_text.len() < 5 {
                            throw_input.angle_text.push(character);
                        }
                    }
                    crate::model::ThrowField::Velocity => {
                        if throw_input.velocity_text.len() < 5 {
                            throw_input.velocity_text.push(character);
                        }
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            match throw_input.focus {
                crate::model::ThrowField::Angle => {
                    throw_input.angle_text.pop();
                }
                crate::model::ThrowField::Velocity => {
                    throw_input.velocity_text.pop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Settings;

    #[test]
    fn volume_cycle_wraps() {
        assert_eq!(cycle_volume(1.0), 0.0);
        assert_eq!(cycle_volume(0.0), 0.2);
    }

    #[test]
    fn new_match_respects_target_score() {
        let settings = Settings { target_score: 7, ..Settings::default() };
        let match_state = new_match(&settings);
        assert_eq!(match_state.target_score, 7);
    }

    #[test]
    fn player_two_angle_is_mirrored() {
        let velocity = parse_throw_velocity(PlayerId::Two, "45", "50").unwrap();
        assert!(velocity.x < 0.0);
        assert!(velocity.y < 0.0);

        let player_one = parse_throw_velocity(PlayerId::One, "45", "50").unwrap();
        assert!(player_one.x > 0.0);
        assert!((velocity.y - player_one.y).abs() < f32::EPSILON);
    }
}
