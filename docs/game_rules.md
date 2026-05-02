# Botclimber: Game Rules and Physics

This document provides the technical specifications for the Botclimber game engine. Bot developers should use these constants and rules to calibrate their movement algorithms.

## 1. Coordinate System
- **Orientation**: 2D Cartesian coordinate system.
- **X-axis**: Horizontal position. `X = 0` is the center of the tower. Positive values are to the right, negative to the left.
- **Y-axis**: Vertical height. `Y = 0` is the starting ground level. Positive values are upwards (climbing).
- **Units**: Abstract game units are used for all spatial and physical calculations.
- **Tower Width**: The tower has a functional width of 20 units (`X` range from `-10.0` to `10.0`). The tower walls at `X = -10.0` and `X = 10.0` are solid and cause reflections.

## 2. Physics Constants
- **Gravity**: `-10.0 units/s²` applied every tick.
- **Tick Rate**: The server processes physics at 60 Hz (60 ticks per second).
- **Jump Strength**:
    - **Minimum**: `0`
    - **Maximum**: `20.0`
    - **Calculation**: The initial velocity vector `v0` is calculated by multiplying the normalized **direction** vector by the **jump strength** (`v0 = direction_unit_vector * strength`).
- **Direction**:
    - Expressed as a vector `(x, y)` or an angle in degrees (`0` to `360`).
    - If a vector is supplied that is not a unit vector, the server **normalizes it automatically** and accepts the request (no error is returned).
    - `0°` or `(0, 1)` is straight up.
- **Air Resistance**: Currently `1.0` (no air resistance applied per tick).

## 3. Player Figures
- **Dimensions**: Each player is represented as an Axis-Aligned Bounding Box (AABB) with a width of `0.8 units` and a height of `0.8 units`.
- **Collisions**: 
    - **Environment**: Players collide with all surfaces of platforms (top, bottom, and sides) as well as the tower walls.
    - **Physics**: Collisions result in a **reflection** of the player's velocity vector relative to the surface normal of the surface hit (platform side or tower wall). 
    - **Bounce Reduction**: Every reflection applies a flat **friction factor of `0.5`** to the velocity components along the reflected axis. For top/bottom surface hits, both `v_x` and `v_y` are multiplied by `0.5` (with `v_y` sign-flipped). For side hits (left/right of platform or tower wall), only `v_x` is multiplied by `-0.5`.
    - **Players**: Players are "ghosts" relative to each other; they do not collide or block each other's movement.
- **Landing**: A player is considered in the **LANDED** state if their vertical velocity is sufficiently low (e.g., `|v_y| < 0.1 units/s`) upon hitting a top surface. 
- **Player States**:
    - **LANDED**: The figure is stationary on a platform or the ground. Jump commands are only accepted in this state.
    - **JUMPING**: The figure has any non-zero velocity (e.g., during flight, reflection, or falling).

## 4. Platforms
- **Generation (Prototype)**: For the current prototype, platforms are generated using a **fixed repeating pattern**.
    - The pattern covers a vertical distance of **50 units**.
    - Once a player climbs past 50 units, the same platform configuration repeats (e.g., a platform at `Y=5` will have a counterpart at `Y=55`, `Y=105`, etc.).
- **Dimensions**: 
    - Width: Fixed within the pattern (variable between `2.0` and `5.0` units).
    - Thickness: Constant `0.5` units.
- **Surface**: All surfaces (top, bottom, left, right) are solid and cause reflections.

## 5. Death and Reset
- **Falling (Death Condition)**: If a player's `Y` coordinate falls more than `15.0` units below their current session's "Maximal Height Reached", they are considered to have "fallen off".
- **Game Modes**:
    - **Checkpoint Mode (Default)**: The server is set to this mode by the Administrator. Upon falling off, the player is reset to the last position where they were in the **LANDED** state. If no such position exists (e.g., first jump), they reset to `Y = 0`.
    - **Hardcore Mode**: The server is set to this mode by the Administrator. In this mode, players are not reset when they fall. They simply continue to fall until they land on a platform or the ground. The challenge is greater as any mistake results in a loss of height and momentum without a safety net.
- **Velocity Reset**: Upon a system-initiated reset (e.g., in Checkpoint Mode), all velocity is set to `0.0`.

## 6. Interaction Rules
- **Join**: Spawns the player at `(0, 0)`.
- **Jump**: Can only be executed when the player is in the **LANDED** state. Jumps requested while in the JUMPING state are ignored.
- **Leave**: Removes the player from the active session.
