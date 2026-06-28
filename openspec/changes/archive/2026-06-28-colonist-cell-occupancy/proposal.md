## Why

Colonists currently snap onto the same grid cell without collision checks — multiple units can overlap when paths converge or when eating from adjacent stand tiles. A global settled-cell occupancy rule makes movement readable and consistent with colony-sim expectations: units pass through each other while moving but never share a cell when stopped.

## What Changes

- At most one colonist may occupy a grid cell when movement completes a waypoint step (settled position)
- Colonists may pass through each other during movement interpolation; pathfinding does not treat other colonists as obstacles
- If a target cell is occupied at snap time, the colonist waits at its current position and does not advance its path
- Eat task assignment prefers unoccupied adjacent stand tiles; idle colonists on a stand tile block that cell (no exceptions)
- No client or snapshot protocol changes

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Add unique settled-cell occupancy requirement; update eat assignment to skip occupied stand tiles; pathfinding unchanged regarding colonist blocking

## Impact

- `packages/game-core`: two-phase `colonist_movement` with occupancy check at waypoint snap; occupancy helper; filter occupied cells in `nearest_eat_assignment` / `best_adjacent_stand`
- `packages/client`: no changes (visual overlap resolves naturally once sim enforces occupancy)
