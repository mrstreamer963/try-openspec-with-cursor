## Why

When a colonist's Food need is critical but no BerryBush is available, `auto_assign_tasks` tries Eat and gives up — it never falls through to Sleep even if Sleep is also critical and a free Bed exists. Colonists stay idle (or keep wandering) while both needs hit zero, which breaks expected survival behavior.

## What Changes

- When the highest-priority critical need (Food before Sleep) cannot be assigned, try the next critical need that has a satisfiable target and path
- Clear any active wander path when a critical need assignment succeeds (Eat or Sleep)
- Add spec scenarios for unsatisfiable Food fallback to Sleep, and symmetric Sleep→Food when no bed is available
- No new task types, components, or snapshot/client changes

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Extend automatic task assignment with fallback when the primary critical need has no satisfiable target; clear wander path on need assignment

## Impact

- `packages/game-core`: refactor need-selection logic in `auto_assign_tasks` (`systems.rs`); add unit tests for fallback cases
- `packages/client`: no changes
- Complements `idle-wander` (wander must not block or mask critical need fallback)
