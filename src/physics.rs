use crate::assets;
use crate::model::{Building, GorillaState, ProjectileState, RoundState, WindState};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collision {
    None,
    Building,
    Gorilla(usize),
    OutOfBounds,
}

pub fn step_projectile(projectile: &mut ProjectileState, wind: WindState, gravity: f32, dt: f32) {
    projectile.velocity.x += wind.strength * dt * 0.45;
    projectile.velocity.y += gravity * dt;
    projectile.position += projectile.velocity * dt;
    projectile.trail.push(projectile.position);
    if projectile.trail.len() > 18 {
        projectile.trail.remove(0);
    }
}

pub fn resolve_collision(
    position: Vec2,
    buildings: &[Building],
    gorillas: &[GorillaState; 2],
) -> Collision {
    let far_side_limit = assets::VIRTUAL_WIDTH * 4.0;
    if position.y > assets::VIRTUAL_HEIGHT + 16.0
        || position.x < -far_side_limit
        || position.x > far_side_limit
    {
        return Collision::OutOfBounds;
    }

    for building in buildings {
        if building.rect.contains(position) {
            return Collision::Building;
        }
    }

    for (index, gorilla) in gorillas.iter().enumerate() {
        if gorilla.position.distance(position) <= assets::GORILLA_RADIUS + assets::BANANA_RADIUS {
            return Collision::Gorilla(index);
        }
    }

    Collision::None
}

pub fn trajectory_preview(round: &RoundState, start: Vec2, velocity: Vec2) -> Vec<Vec2> {
    let mut preview = ProjectileState {
        position: start,
        velocity,
        trail: Vec::new(),
        alive: true,
    };
    let mut points = Vec::new();
    for _ in 0..assets::TRAJECTORY_STEPS {
        step_projectile(&mut preview, round.wind, round.gravity, assets::TRAJECTORY_DT);
        if !matches!(
            resolve_collision(preview.position, &round.city.buildings, &round.gorillas),
            Collision::None
        ) {
            break;
        }
        points.push(preview.position);
    }
    points
}

pub fn winner_for_hit(hit_index: usize) -> usize {
    match hit_index {
        0 => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_building() -> Building {
        Building {
            rect: Rect::new(100.0, 100.0, 20.0, 80.0),
            windows: Vec::new(),
        }
    }

    fn sample_gorillas() -> [GorillaState; 2] {
        [
            GorillaState {
                player: crate::model::PlayerId::One,
                position: vec2(40.0, 80.0),
                arm_angle: 0.0,
                celebrating: 0.0,
            },
            GorillaState {
                player: crate::model::PlayerId::Two,
                position: vec2(250.0, 80.0),
                arm_angle: std::f32::consts::PI,
                celebrating: 0.0,
            },
        ]
    }

    #[test]
    fn projectile_accumulates_gravity() {
        let mut projectile = ProjectileState {
            position: vec2(10.0, 10.0),
            velocity: vec2(20.0, -5.0),
            trail: Vec::new(),
            alive: true,
        };
        step_projectile(&mut projectile, WindState { strength: 0.0 }, 30.0, 0.5);
        assert!(projectile.velocity.y > 0.0);
        assert!(projectile.position.x > 10.0);
    }

    #[test]
    fn projectile_receives_wind_drift() {
        let mut projectile = ProjectileState {
            position: vec2(10.0, 10.0),
            velocity: vec2(0.0, 0.0),
            trail: Vec::new(),
            alive: true,
        };
        step_projectile(&mut projectile, WindState { strength: 20.0 }, 0.0, 1.0);
        assert!(projectile.velocity.x > 0.0);
    }

    #[test]
    fn detects_building_collision() {
        let collision = resolve_collision(vec2(105.0, 120.0), &[sample_building()], &sample_gorillas());
        assert_eq!(collision, Collision::Building);
    }

    #[test]
    fn detects_gorilla_hit() {
        let collision = resolve_collision(vec2(40.0, 80.0), &[sample_building()], &sample_gorillas());
        assert_eq!(collision, Collision::Gorilla(0));
    }

    #[test]
    fn detects_out_of_bounds() {
        let collision = resolve_collision(
            vec2(120.0, assets::VIRTUAL_HEIGHT + 20.0),
            &[sample_building()],
            &sample_gorillas(),
        );
        assert_eq!(collision, Collision::OutOfBounds);
    }

    #[test]
    fn allows_shot_to_leave_side_and_continue() {
        let collision = resolve_collision(vec2(-10.0, 40.0), &[sample_building()], &sample_gorillas());
        assert_eq!(collision, Collision::None);
    }
}
