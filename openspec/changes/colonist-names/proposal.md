## Why

Colonists are currently identified only by numeric IDs (`#1`, `#2`, `#3`), which feel impersonal and are hard to track while watching the simulation. Display names make colonists recognizable at a glance and improve the info panel when a colonist is selected.

## What Changes

- Each colonist spawns with a unique display name drawn from a fixed name pool
- `ColonistSnapshot` gains a `name` field; numeric `id` remains for internal identity
- PixiJS renders the colonist name above the sprite at all times
- Colonist info panel shows `Name (#id)` instead of `Colonist #id`
- Player renaming is out of scope for this change

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Colonists receive unique display names at spawn; snapshot exposes `name`
- `view-layer`: Name labels render above colonist sprites; info panel shows name and id together

## Impact

- `packages/game-core`: new `ColonistName` component, name assignment in `spawn_colonists`, `ColonistSnapshot.name`
- `packages/client`: `ColonistSnapshot.name` type, `PixiRenderer` name labels, `ColonistInfo.vue` header
- **Protocol**: Snapshot JSON adds `name` string per colonist (additive; `id` unchanged)
