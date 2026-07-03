## Why

The colonist info panel shows Food and Sleep numeric values but not their critical status. When a colonist's Food reaches 0, the player sees an empty bar and "0" but no indication that the colonist is hungry — the same gap exists for sleep. This makes it hard to understand why a colonist is idle or wandering instead of eating.

## What Changes

- `ColonistSnapshot` gains `hungry` and `wants_sleep` boolean fields derived from the same threshold logic as simulation buffs
- Colonist info panel displays critical need status labels (e.g. "Hungry", "Wants sleep") alongside need bars and values
- Critical need rows receive visual emphasis (color/warning styling) when active

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Snapshot exposes critical need status flags per colonist
- `view-layer`: Info panel shows need status labels and highlights critical needs

## Impact

- `packages/game-core`: `ColonistSnapshot` fields, snapshot builder reads `Hungry` / `WantsSleep` components
- `packages/client`: TypeScript types, `ColonistInfo.vue` need status display
- **Protocol**: Snapshot JSON adds `hungry` and `wants_sleep` booleans per colonist (additive)
