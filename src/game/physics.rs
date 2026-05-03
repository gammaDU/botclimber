/// Physics constants from game_rules.md
pub const GRAVITY: f64 = -10.0;
pub const TICK_RATE_HZ: f64 = 60.0;
pub const DT: f64 = 1.0 / TICK_RATE_HZ;

pub const TOWER_WIDTH: f64 = 20.0;
pub const AIR_RESISTANCE: f64 = 1.0;
pub const FRICTION: f64 = 0.5;

pub const PLAYER_WIDTH: f64 = 0.8;
pub const PLAYER_HEIGHT: f64 = 0.8;

pub const JUMP_STRENGTH_MIN: f64 = 0.0;
pub const JUMP_STRENGTH_MAX: f64 = 20.0;

pub const DEATH_DROP: f64 = 15.0;

use crate::models::{Platform, PlayerMovementState, PlayerState, Rect, Vector2};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("Invalid jump strength, must be between {JUMP_STRENGTH_MIN} and {JUMP_STRENGTH_MAX}")]
    InvalidJumpStrength,
    #[error("Jump direction can not be the null vector")]
    InvalidJumpDirection,
    #[error("Player is in the air")]
    InvalidJumpState,
}

/// Apply one physics tick to a single player.
/// Returns `true` if the player died (fell more than DEATH_DROP below session peak) and should be reset.
pub fn tick_player(player: &mut PlayerState, platforms: &[Platform]) -> bool {
    // Apply gravity
    player.velocity.y += GRAVITY * DT;
    player.velocity *= AIR_RESISTANCE;

    // Integrate position
    player.position += player.velocity * DT;

    // AABB collision with platforms
    let player_rect = Rect {
        x: player.position.x - PLAYER_WIDTH / 2.0,
        y: player.position.y,
        width: PLAYER_WIDTH,
        height: PLAYER_HEIGHT,
    };

    let mut grounded = false;
    for platform in platforms {
        if let Some(side) = aabb_overlap(&platform.rect, &player_rect) {
            //bounce
            match side {
                OverlapSide::Top | OverlapSide::Bottom => {
                    player.velocity.x *= FRICTION;
                    player.velocity.y *= -FRICTION;
                }
                OverlapSide::Left | OverlapSide::Right => {
                    player.velocity.x *= -FRICTION;
                }
            }
            //place outside boundry
            match side {
                OverlapSide::Top => {
                    grounded = true;
                    player.position.y = platform.rect.y + platform.rect.height;
                }
                OverlapSide::Bottom => {
                    player.position.y = platform.rect.y - PLAYER_HEIGHT;
                }
                OverlapSide::Left => {
                    player.position.x = platform.rect.x - PLAYER_WIDTH / 2.0;
                }
                OverlapSide::Right => {
                    player.position.x = platform.rect.x + platform.rect.width + PLAYER_WIDTH / 2.0;
                }
            }
            if side == OverlapSide::Top {
                grounded = true;
                player.position.y = platform.rect.y + platform.rect.height;
            }
        }
    }

    //bounce of tower walls
    if player_rect.x < -TOWER_WIDTH / 2.0 || player_rect.x + player_rect.width > TOWER_WIDTH / 2.0 {
        player.velocity.x *= -FRICTION;
        player.position.x = player.position.x.clamp(
            (-TOWER_WIDTH + PLAYER_WIDTH) / 2.0,
            (TOWER_WIDTH - PLAYER_WIDTH) / 2.0,
        );
    }

    //bounce of floor
    if player_rect.y < 0.0 {
        player.velocity.x *= FRICTION;
        player.velocity.y *= -FRICTION;
        player.position.y = 0.0;
        grounded = true;
    }

    //set state to "Landed" if (little to) no vertical movement
    if grounded && player.velocity.y.abs() < 0.1 {
        player.state = PlayerMovementState::Landed;
        player.velocity *= 0.0;
    }

    // Update session max height
    if player.position.y > player.session_max_height {
        player.session_max_height = player.position.y;
    }

    // Death check: player fell more than DEATH_DROP below their session peak
    player.state == PlayerMovementState::Landed
        && player.position.y < player.session_max_height - DEATH_DROP
}

/// Apply a jump command to a player.
/// `direction` need not be a unit vector — it is normalised here.
/// Returns `Err` if the vector is zero or strength is out of range.
pub fn apply_jump(
    player: &mut PlayerState,
    direction: Vector2,
    strength: f64,
) -> Result<(), PhysicsError> {
    if strength < JUMP_STRENGTH_MIN || strength > JUMP_STRENGTH_MAX {
        return Err(PhysicsError::InvalidJumpStrength);
    }
    if player.state == PlayerMovementState::Jumping {
        return Err(PhysicsError::InvalidJumpState);
    }
    let unit = direction
        .normalize()
        .ok_or(PhysicsError::InvalidJumpDirection)?;
    player.velocity = Vector2 {
        x: unit.x * strength,
        y: unit.y * strength,
    };
    player.state = PlayerMovementState::Jumping;
    Ok(())
}

/// Which side of rectangle `a` overlaps rectangle `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlapSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Axis-aligned bounding box overlap test.
/// Returns the side of `a` through which the smallest penetration occurs,
/// or `None` if the rectangles do not overlap.
pub fn aabb_overlap(a: &Rect, b: &Rect) -> Option<OverlapSide> {
    let overlap_x = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
    let overlap_y = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None;
    }

    if overlap_x < overlap_y {
        // Horizontal axis has smaller penetration
        if a.x + a.width / 2.0 < b.x + b.width / 2.0 {
            Some(OverlapSide::Right)
        } else {
            Some(OverlapSide::Left)
        }
    } else {
        // Vertical axis has smaller penetration
        if a.y + a.height / 2.0 < b.y + b.height / 2.0 {
            Some(OverlapSide::Top)
        } else {
            Some(OverlapSide::Bottom)
        }
    }
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
            player_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "test".into(),
            position: Vector2::zero(),
            velocity: Vector2::zero(),
            state: PlayerMovementState::Landed,
            session_max_height: 5.0,
            activity_rating: 0.0,
            last_heartbeat: Utc::now(),
            last_landed_position: Vector2::zero(),
        }
    }

    fn make_platform(x: f64, y: f64, w: f64, h: f64) -> Platform {
        Platform {
            id: Uuid::new_v4(),
            rect: Rect {
                x,
                y,
                width: w,
                height: h,
            },
        }
    }

    #[test]
    fn gravity_accelerates_downward() {
        let mut p = make_player();
        p.position = Vector2::new(0.0, 5.0);
        tick_player(&mut p, &[]);
        assert!(p.velocity.y < 0.0, "gravity should pull velocity negative");
        assert_eq!(
            p.velocity.y,
            (GRAVITY * DT),
            "gravity should equal DT * GRAVITY"
        );
        assert!(p.position.y < 5.0, "position should be updated");
    }

    #[test]
    fn jump_normalises_non_unit_direction() {
        let mut p = make_player();
        // (3, 4) has length 5 — not a unit vector
        apply_jump(&mut p, Vector2::new(3.0, 4.0), 10.0).unwrap();
        let speed = p.velocity.length();
        assert!(
            (speed - 10.0).abs() < 1e-9,
            "speed should equal strength after normalisation"
        );
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
        p.state = PlayerMovementState::Jumping;
        // Platform: y=0, height=1 → top surface at y=1.
        let platform = make_platform(-1.0, 0.0, 4.0, 1.0);

        tick_player(&mut p, &[platform.clone()]);
        assert_eq!(p.state, PlayerMovementState::Jumping);
        assert_eq!(p.velocity.y, 2.5 - (GRAVITY * DT * 0.5));
    }

    #[test]
    fn death_below_threshold() {
        let mut p = make_player();
        p.session_max_height = 16.0;
        p.state = PlayerMovementState::Jumping;
        p.position = Vector2::new(0.0, 5.0);
        p.velocity = Vector2::new(0.0, -5.0);
        let mut ctr = 0;
        loop {
            if tick_player(&mut p, &[]) {
                break;
            }
            ctr += 1;
        }
        assert!(ctr > 1);
        assert_eq!(p.state, PlayerMovementState::Landed);
    }

    #[test]
    fn aabb_overlap_detects_intersection() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 2.0,
        };
        let b = Rect {
            x: 1.0,
            y: 1.0,
            width: 2.0,
            height: 2.0,
        };
        assert_eq!(aabb_overlap(&a, &b), Some(OverlapSide::Top));
    }

    #[test]
    fn aabb_no_overlap_adjacent() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
        let b = Rect {
            x: 1.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
        assert_eq!(aabb_overlap(&a, &b), None);
    }
}
