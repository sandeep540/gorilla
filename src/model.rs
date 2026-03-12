use crate::assets;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerId {
    One,
    Two,
}

impl PlayerId {
    pub fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Player 1",
            Self::Two => "Player 2",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub target_score: u32,
    pub trajectory_preview: bool,
    pub screen_shake: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            master_volume: 0.8,
            sfx_volume: 0.8,
            target_score: 3,
            trajectory_preview: true,
            screen_shake: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub name: String,
    pub score: u32,
}

#[derive(Clone, Debug)]
pub struct GorillaState {
    pub player: PlayerId,
    pub position: Vec2,
    pub arm_angle: f32,
    pub celebrating: f32,
}

#[derive(Clone, Debug)]
pub struct ProjectileState {
    pub position: Vec2,
    pub velocity: Vec2,
    pub trail: Vec<Vec2>,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct WindState {
    pub strength: f32,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub rect: Rect,
    pub windows: Vec<(Rect, bool)>,
}

#[derive(Clone, Debug)]
pub struct CityState {
    pub buildings: Vec<Building>,
    pub skyline_seed: u64,
}

#[derive(Clone, Debug)]
pub struct Explosion {
    pub position: Vec2,
    pub timer: f32,
    pub max_radius: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ThrowInput {
    pub angle_text: String,
    pub velocity_text: String,
    pub focus: ThrowField,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThrowField {
    #[default]
    Angle,
    Velocity,
}

#[derive(Clone, Debug)]
pub struct RoundState {
    pub city: CityState,
    pub gorillas: [GorillaState; 2],
    pub active_player: PlayerId,
    pub wind: WindState,
    pub gravity: f32,
    pub projectile: Option<ProjectileState>,
    pub explosion: Option<Explosion>,
    pub winner: Option<PlayerId>,
    pub banner_timer: f32,
    pub throw_input: ThrowInput,
}

#[derive(Clone, Debug)]
pub struct MatchState {
    pub players: [PlayerState; 2],
    pub round: RoundState,
    pub target_score: u32,
    pub round_number: u32,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub settings: Settings,
    pub match_state: MatchState,
}

pub fn new_match(settings: &Settings) -> MatchState {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1);
    let mut rng = SmallRng::seed_from_u64(seed);
    MatchState {
        players: [
            PlayerState {
                name: "Player 1".to_string(),
                score: 0,
            },
            PlayerState {
                name: "Player 2".to_string(),
                score: 0,
            },
        ],
        round: spawn_round(&mut rng),
        target_score: settings.target_score,
        round_number: 1,
    }
}

pub fn reset_scores(state: &mut MatchState, settings: &Settings) {
    for player in &mut state.players {
        player.score = 0;
    }
    state.target_score = settings.target_score;
    state.round_number = 1;
}

pub fn spawn_round(rng: &mut SmallRng) -> RoundState {
    let skyline_seed = rng.random::<u64>();
    let mut city_rng = SmallRng::seed_from_u64(skyline_seed);
    let mut buildings = Vec::new();
    let count = (assets::VIRTUAL_WIDTH / assets::BUILDING_WIDTH) as usize;
    let min_height = assets::VIRTUAL_HEIGHT * 0.30;
    let max_height = assets::VIRTUAL_HEIGHT * 0.58;
    let mut previous_height = city_rng.random_range(min_height..max_height);

    for index in 0..count {
        let width = assets::BUILDING_WIDTH;
        let step = city_rng.random_range(-22.0..22.0);
        let blended = previous_height + step;
        let height = blended.clamp(min_height, max_height);
        previous_height = height;
        let x = index as f32 * width;
        let y = assets::VIRTUAL_HEIGHT - height;
        let rect = Rect::new(x, y, width, height);
        let mut windows = Vec::new();
        let cols = 3;
        let rows = ((height - 10.0) / 10.0).max(1.0) as usize;
        for row in 0..rows {
            for col in 0..cols {
                let wx = x + 4.0 + col as f32 * 6.0;
                let wy = y + 5.0 + row as f32 * 10.0;
                windows.push((Rect::new(wx, wy, 3.0, 5.0), city_rng.random_bool(0.55)));
            }
        }
        buildings.push(Building { rect, windows });
    }

    let (left_index, right_index) = select_balanced_spawn_pair(&buildings, &mut city_rng);
    let left_building = &buildings[left_index];
    let right_building = &buildings[right_index];
    let gorillas = [
        GorillaState {
            player: PlayerId::One,
            position: vec2(
                left_building.rect.x + left_building.rect.w / 2.0,
                left_building.rect.y - assets::GORILLA_RADIUS - 1.0,
            ),
            arm_angle: 0.0,
            celebrating: 0.0,
        },
        GorillaState {
            player: PlayerId::Two,
            position: vec2(
                right_building.rect.x + right_building.rect.w / 2.0,
                right_building.rect.y - assets::GORILLA_RADIUS - 1.0,
            ),
            arm_angle: std::f32::consts::PI,
            celebrating: 0.0,
        },
    ];

    RoundState {
        city: CityState {
            buildings,
            skyline_seed,
        },
        gorillas,
        active_player: PlayerId::One,
        wind: WindState {
            strength: city_rng.random_range(-20.0..20.0),
        },
        gravity: 30.0,
        projectile: None,
        explosion: None,
        winner: None,
        banner_timer: assets::ROUND_BANNER_TIME,
        throw_input: default_throw_input(PlayerId::One),
    }
}

fn select_balanced_spawn_pair(buildings: &[Building], rng: &mut SmallRng) -> (usize, usize) {
    let count = buildings.len();
    let left_candidates = 1..(count / 3).max(2);
    let right_start = ((count * 2) / 3).min(count.saturating_sub(2));
    let right_candidates = right_start..count - 1;

    let mut best_pair = (1usize, count.saturating_sub(2));
    let mut best_score = f32::MAX;

    for left_index in left_candidates {
        for right_index in right_candidates.clone() {
            let left_height = buildings[left_index].rect.h;
            let right_height = buildings[right_index].rect.h;
            let height_delta = (left_height - right_height).abs();

            let left_cover = neighbor_cover(buildings, left_index);
            let right_cover = neighbor_cover(buildings, right_index);
            let cover_delta = (left_cover - right_cover).abs();

            let score = height_delta + cover_delta * 0.8;
            if score < best_score
                || ((score - best_score).abs() < f32::EPSILON && rng.random_bool(0.5))
            {
                best_score = score;
                best_pair = (left_index, right_index);
            }
        }
    }

    best_pair
}

fn neighbor_cover(buildings: &[Building], index: usize) -> f32 {
    let current_height = buildings[index].rect.h;
    let mut cover = 0.0;

    let start = index.saturating_sub(1);
    let end = (index + 1).min(buildings.len() - 1);
    for (neighbor, building) in buildings.iter().enumerate().take(end + 1).skip(start) {
        if neighbor == index {
            continue;
        }
        let delta = building.rect.h - current_height;
        if delta > 0.0 {
            cover += delta;
        }
    }

    cover
}

pub fn start_next_round(match_state: &mut MatchState) {
    let mut rng = SmallRng::seed_from_u64(match_state.round.city.skyline_seed ^ (match_state.round_number as u64 + 17));
    match_state.round = spawn_round(&mut rng);
    match_state.round_number += 1;
}

pub fn hand_position(gorilla: &GorillaState) -> Vec2 {
    gorilla.position + vec2(gorilla.arm_angle.cos(), gorilla.arm_angle.sin()) * (assets::GORILLA_RADIUS + 3.0)
}

pub fn default_throw_input(_player: PlayerId) -> ThrowInput {
    ThrowInput {
        angle_text: String::new(),
        velocity_text: String::new(),
        focus: ThrowField::Angle,
    }
}

use macroquad::prelude::{vec2, Rect, Vec2};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_spawns_valid_gorillas_and_wind() {
        let mut rng = SmallRng::seed_from_u64(7);
        let round = spawn_round(&mut rng);
        assert!(round.wind.strength >= -20.0);
        assert!(round.wind.strength <= 20.0);
        assert!(round.gorillas[0].position.x < round.gorillas[1].position.x);
        for building in &round.city.buildings {
            assert!(building.rect.h >= assets::VIRTUAL_HEIGHT * 0.30);
            assert!(building.rect.h <= assets::VIRTUAL_HEIGHT * 0.58);
        }
        let left_height = round.city.buildings
            .iter()
            .find(|building| (building.rect.x..=building.rect.x + building.rect.w).contains(&round.gorillas[0].position.x))
            .unwrap()
            .rect
            .h;
        let right_height = round.city.buildings
            .iter()
            .find(|building| (building.rect.x..=building.rect.x + building.rect.w).contains(&round.gorillas[1].position.x))
            .unwrap()
            .rect
            .h;
        assert!((left_height - right_height).abs() <= 24.0);
    }

    #[test]
    fn reset_scores_clears_scores() {
        let settings = Settings::default();
        let mut match_state = new_match(&settings);
        match_state.players[0].score = 2;
        match_state.players[1].score = 1;
        reset_scores(&mut match_state, &settings);
        assert_eq!(match_state.players[0].score, 0);
        assert_eq!(match_state.players[1].score, 0);
        assert_eq!(match_state.round_number, 1);
    }
}
