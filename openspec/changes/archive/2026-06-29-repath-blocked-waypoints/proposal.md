## Why

Colonists with Eat, Build, or Sleep tasks wait indefinitely when their next path waypoint is occupied by another colonist. Only idle wander paths are cleared and reassigned today, causing visible deadlocks (e.g. a builder stuck next to an eater blocking the stand tile).

## What Changes

- Replace `release_blocked_idle_wander` with `repath_blocked_colonists` that handles all task kinds
- When the next waypoint is occupied: Idle clears path for wander reassignment; Eat/Build/Sleep repath to `task.target` via `find_path_for_colonist`
- When repath fails: clear task and release reservations (Eat also prefers Sleep on next assignment pass)
- When repath returns the same route (goal cell occupied): hold position without churn

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Blocked waypoint repath for all active tasks

## Impact

- `packages/game-core`: `systems.rs` — repath helper, updated `auto_assign_tasks`
- `packages/client`: no changes
