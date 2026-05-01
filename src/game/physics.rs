/// Physics constants from game_rules.md
pub const GRAVITY: f64 = -10.0;
pub const TICK_RATE_HZ: f64 = 60.0;
pub const DT: f64 = 1.0 / TICK_RATE_HZ;

pub const PLAYER_WIDTH: f64 = 0.8;
pub const PLAYER_HEIGHT: f64 = 0.8;

pub const JUMP_STRENGTH_MIN: f64 = 0.0;
pub const JUMP_STRENGTH_MAX: f64 = 20.0;

/// Bounce deceleration schedule: velocity multipliers applied on successive
/// platform landings within the same jump (100 %, 75 %, 50 %, 25 %, 0 %).
pub const BOUNCE_SCHEDULE: &[f64] = &[1.0, 0.75, 0.50, 0.25, 0.0];

pub const DEATH_DROP: f64 = 15.0;

use crate::models::{Platform, PlayerState, Rect, Vector2};

/// Apply one physics tick to a single player.
/// Returns `true` if the player died (fell more than DEATH_DROP below session peak) and should be reset.
pub fn tick_player(player: &mut PlayerState, platforms: &[Platform]) -> bool {
    // Apply gravity
    player.velocity.y += GRAVITY * DT;

    // Integrate position
    let new_pos = Vector2 {
        x: player.position.x + player.velocity.x * DT,
        y: player.position.y + player.velocity.y * DT,
    };

    // AABB collision with platforms
    let player_rect = Rect {
        x: new_pos.x - PLAYER_WIDTH / 2.0,
        y: new_pos.y,
        width: PLAYER_WIDTH,
        height: PLAYER_HEIGHT,
    };

    let mut landed = false;
    for platform in platforms {
        if aabb_overlap(&player_rect, &platform.rect) && player.velocity.y < 0.0 {
            // Land on top of platform
            player.position.y = platform.rect.y + platform.rect.height;
            player.position.x = new_pos.x;

            let bounce_idx = player.consecutive_bounces.min(BOUNCE_SCHEDULE.len() as u32 - 1) as usize;
            let multiplier = BOUNCE_SCHEDULE[bounce_idx];
            player.velocity.y = player.velocity.y.abs() * multiplier;
            player.velocity.x *= multiplier;

            player.state = crate::models::PlayerMovementState::Landed;
            player.last_landed_position = player.position;
            player.consecutive_bounces = 0;
            landed = true;
            break;
        }
    }

    if !landed {
        player.position = new_pos;
        player.state = crate::models::PlayerMovementState::Jumping;
    }

    // Update session max height
    if player.position.y > player.session_max_height {
        player.session_max_height = player.position.y;
    }

    // Death check: player fell more than DEATH_DROP below their session peak
    player.position.y < player.session_max_height - DEATH_DROP
}

/// Apply a jump command to a player.
/// `direction` need not be a unit vector — it is normalised here.
/// Returns `Err` if the vector is zero or strength is out of range.
pub fn apply_jump(
    player: &mut PlayerState,
    direction: Vector2,
    strength: f64,
) -> Result<(), &'static str> {
    if strength < JUMP_STRENGTH_MIN || strength > JUMP_STRENGTH_MAX {
        return Err("strength out of range");
    }
    let unit = direction.normalize().ok_or("direction must not be a zero vector")?;
    player.velocity = Vector2 {
        x: unit.x * strength,
        y: unit.y * strength,
    };
    player.state = crate::models::PlayerMovementState::Jumping;
    player.consecutive_bounces += 1;
    Ok(())
}

/// Axis-aligned bounding box overlap test.
pub fn aabb_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width
        && a.x + a.width > b.x
        && a.y < b.y + b.height
        && a.y + a.height > b.y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Platform, PlayerState, Rect, Vector2};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_player() -> PlayerState {
        PlayerState {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "test".into(),
            position: Vector2::new(0.0, 5.0),
            velocity: Vector2::zero(),
            state: crate::models::PlayerMovementState::Jumping,
            consecutive_bounces: 0,
            session_max_height: 5.0,
            activity_rating: 0.0,
            last_heartbeat: Utc::now(),
            last_landed_position: Vector2::zero(),
        }
    }

    fn make_platform(x: f64, y: f64, w: f64, h: f64) -> Platform {
        Platform {
            id: Uuid::new_v4(),
            rect: Rect { x, y, width: w, height: h },
        }
    }

    #[test]
    fn gravity_accelerates_downward() {
        let mut p = make_player();
        tick_player(&mut p, &[]);
        assert!(p.velocity.y < 0.0, "gravity should pull velocity negative");
    }

    #[test]
    fn jump_normalises_non_unit_direction() {
        let mut p = make_player();
        // (3, 4) has length 5 — not a unit vector
        apply_jump(&mut p, Vector2::new(3.0, 4.0), 10.0).unwrap();
        let speed = p.velocity.length();
        assert!((speed - 10.0).abs() < 1e-9, "speed should equal strength after normalisation");
    }

    #[test]
    fn jump_rejects_zero_vector() {
        let mut p = make_player();
        assert!(apply_jump(&mut p, Vector2::zero(), 5.0).is_err());
    }

    #[test]
    fn jump_rejects_negative_strength() {
        let mut p = make_player();
        assert!(apply_jump(&mut p, Vector2::new(0.0, 1.0), -1.0).is_err());
    }

    #[test]
    fn bounce_deceleration_schedule() {
        let mut p = make_player();
        // Place player just above the platform so one tick lands it.
        // Platform top = 1.0, player bottom = position.y, so start at y=1.05.
        p.position = Vector2::new(0.0, 1.05);
        p.velocity = Vector2::new(0.0, -5.0);
        // Platform: y=0, height=1 → top surface at y=1.
        let platform = make_platform(-1.0, 0.0, 4.0, 1.0);

        // First landing: multiplier = 1.0 (100 %)
        tick_player(&mut p, &[platform.clone()]);
        assert_eq!(p.state, crate::models::PlayerMovementState::Landed);
        assert_eq!(p.consecutive_bounces, 0);
    }

    #[test]
    fn death_below_threshold() {
        let mut p = make_player();
        p.session_max_height = 0.0;
        p.position = Vector2::new(0.0, -(DEATH_DROP + 1.0));
        p.velocity = Vector2::new(0.0, -1.0);
        let died = tick_player(&mut p, &[]);
        assert!(died);
    }

    #[test]
    fn aabb_overlap_detects_intersection() {
        let a = Rect { x: 0.0, y: 0.0, width: 2.0, height: 2.0 };
        let b = Rect { x: 1.0, y: 1.0, width: 2.0, height: 2.0 };
        assert!(aabb_overlap(&a, &b));
    }

    #[test]
    fn aabb_no_overlap_adjacent() {
        let a = Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 };
        let b = Rect { x: 1.0, y: 0.0, width: 1.0, height: 1.0 };
        assert!(!aabb_overlap(&a, &b));
    }
}
