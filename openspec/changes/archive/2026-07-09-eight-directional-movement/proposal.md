## Why

Colonist pathfinding currently explores only the four cardinal directions, so routes take L-shaped detours around obstacles and look unnatural. Eight-directional movement lets colonists cut diagonally across open tiles, producing shorter, more natural paths while the existing continuous movement system already supports fractional travel between waypoints.

## What Changes

- Extend A* pathfinding from 4 to 8 directions (cardinal + diagonal neighbors)
- Use higher movement cost for diagonal steps (√2) so orthogonal routes remain preferred when equivalent
- Block diagonal moves that would cut through a corner blocked by impassable tiles (no corner-cutting)
- Switch the A* heuristic to octile distance so search remains admissible with mixed step costs
- Keep building interaction stand tiles orthogonal-only (eat, build, deconstruct still use adjacent cardinal cells)
- Add pathfinding tests covering diagonal routes and corner-cutting rules

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: A* pathfinding requirement — paths may include diagonal waypoints; diagonal step cost and corner-cutting rules are specified

## Impact

- **Rust (`packages/game-core`)**: `pathfinding.rs` (neighbor expansion, step costs, heuristic, corner-cutting); pathfinding unit tests in `systems.rs` or `pathfinding.rs`
- **Specs**: delta update to `openspec/specs/colonist-simulation/spec.md`
- **No client changes**: rendering already uses float positions; diagonal movement is visually correct
- **No breaking API changes**: snapshot format and event protocol unchanged
