## Why

Idle colonists currently stand still when they have no critical needs and no construction orders. The colony looks frozen even though simulation is running. Ambient wandering makes the world feel alive and naturally clears colonists from tiles they no longer need (e.g. eat stand tiles after eating).

## What Changes

- When a colonist is idle with no Eat, Sleep, or Build assignment, the simulation assigns a path to a random nearby walkable cell
- On arrival at the wander destination, a new random nearby destination is assigned on the next assignment pass
- Wander uses existing `TaskKind::Idle` and `Path` — no new task type or snapshot fields
- Eat, Sleep, and Build assignments preempt wander immediately (path cleared, task upgraded)
- Wander target selection excludes the colonist's current cell and prefers unoccupied cells when cell occupancy is enforced
- No client or snapshot protocol changes

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Replace "remains idle with no assigned task" standing behavior with idle wander; add wander assignment, preemption, and arrival rules

## Impact

- `packages/game-core`: extend `auto_assign_tasks` with idle wander path assignment; helper to pick random nearby walkable cell; reuse `find_path` and existing movement
- `packages/client`: no changes (wander appears as normal movement while task stays `Idle`)
- Complements `colonist-cell-occupancy` (wander helps colonists vacate occupied stand tiles; wander respects occupied cells when selecting targets if occupancy is implemented)
