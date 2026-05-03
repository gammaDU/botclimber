/// World / platform generation from game_rules.md (US05).
///
/// The prototype uses a fixed repeating pattern:
///   - Platforms are 8 units wide and 1 unit tall.
///   - They are placed every 5 units of height, alternating left/centre/right
///     horizontal positions so a player can always reach the next one.
///   - The pattern repeats indefinitely; new platforms are generated on demand
///     as the highest player climbs.
use uuid::Uuid;

use crate::models::{Platform, Rect};

/// Horizontal positions (centre X) for the three-slot repeating pattern.
const PATTERN_X: &[f64] = &[-8.0, 0.0, 8.0];
const PLATFORM_WIDTH: f64 = 8.0;
const PLATFORM_HEIGHT: f64 = 1.0;
/// Vertical gap between successive platforms.
const VERTICAL_STEP: f64 = 5.0;
/// Number of platforms to keep ahead of the highest player.
const LOOKAHEAD: u32 = 10;

/// Generate the initial set of platforms starting at y = 0.
pub fn initial_platforms() -> Vec<Platform> {
    generate_range(0, LOOKAHEAD)
}

/// Extend the platform list so there are always `LOOKAHEAD` platforms above
/// `highest_y`. Returns any newly created platforms.
pub fn extend_platforms(existing: &[Platform], highest_y: f64) -> Vec<Platform> {
    let top_y = existing
        .iter()
        .map(|p| p.rect.y)
        .fold(f64::NEG_INFINITY, f64::max);

    let needed_top = highest_y + LOOKAHEAD as f64 * VERTICAL_STEP;
    if top_y >= needed_top {
        return Vec::new();
    }

    // Work out which index the current top platform corresponds to.
    let next_index = ((top_y / VERTICAL_STEP).round() as u32) + 1;
    let last_index = ((needed_top / VERTICAL_STEP).ceil() as u32) + 1;

    generate_range(next_index, last_index)
}

fn generate_range(from: u32, to: u32) -> Vec<Platform> {
    (from..to)
        .map(|i| {
            let x_centre = PATTERN_X[(i as usize) % PATTERN_X.len()];
            Platform {
                id: Uuid::new_v4(),
                rect: Rect {
                    x: x_centre - PLATFORM_WIDTH / 2.0,
                    y: i as f64 * VERTICAL_STEP,
                    width: PLATFORM_WIDTH,
                    height: PLATFORM_HEIGHT,
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_platforms_count() {
        let platforms = initial_platforms();
        assert_eq!(platforms.len(), LOOKAHEAD as usize);
    }

    #[test]
    fn platforms_spaced_correctly() {
        let platforms = initial_platforms();
        for (i, p) in platforms.iter().enumerate() {
            let expected_y = i as f64 * VERTICAL_STEP;
            assert!(
                (p.rect.y - expected_y).abs() < 1e-9,
                "platform {i} y={} expected {expected_y}",
                p.rect.y
            );
        }
    }

    #[test]
    fn extend_generates_new_platforms() {
        let existing = initial_platforms();
        let new = extend_platforms(&existing, 30.0);
        assert!(!new.is_empty(), "should generate platforms above y=30");
    }

    #[test]
    fn extend_no_op_when_sufficient() {
        let existing = initial_platforms();
        // initial_platforms covers indices 0..LOOKAHEAD, top platform at y=(LOOKAHEAD-1)*STEP.
        // needed_top = highest_y + LOOKAHEAD*STEP. For no-op we need top_y >= needed_top.
        // top_y = (LOOKAHEAD-1)*VERTICAL_STEP = 45. needed_top = highest_y + 50.
        // So highest_y must be <= -5 for no-op (45 >= highest_y + 50 → highest_y <= -5).
        let new = extend_platforms(&existing, -10.0);
        assert!(
            new.is_empty(),
            "no new platforms needed when lookahead is satisfied"
        );
    }
}
