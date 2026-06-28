## Why

Colonists currently walk onto building tiles to interact — standing on berry bushes looks wrong, and multiple colonists can sleep in the same bed simultaneously. Clear interaction rules (adjacent eating, single-occupant beds) make behavior more intuitive and RimWorld-like.

## What Changes

- **Eat tasks** path to an adjacent walkable tile next to the target BerryBush, not onto the bush tile itself
- Eating succeeds when the colonist is on a stand tile orthogonally adjacent to a bush with berries remaining
- **Sleep tasks** still path to the Bed tile, but only one colonist may use a given bed at a time
- Bed reservation happens at task assignment; occupied beds are skipped when assigning Sleep tasks
- Eat task fails gracefully if no adjacent stand tile exists or the bush is depleted before arrival
- `Task` stores both building coordinates and stand coordinates for interaction tasks

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Adjacent eat interaction, bed single-occupancy, updated task assignment and execution scenarios
- `world-simulation`: Berry bush and bed interaction requirements reflect adjacent eating and bed occupancy

## Impact

- `packages/game-core`: extend `Task` with building/stand coords, add `BedOccupancy` component, update `auto_assign_tasks`, `task_execution`, and pathfinding helpers
- `packages/client`: no required UI changes; colonist positions will naturally appear beside bushes
