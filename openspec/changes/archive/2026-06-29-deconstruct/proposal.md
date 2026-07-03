## Why

Construction is unit-based and reversible only for berry bush auto-removal when depleted. Players cannot remove construction sites or finished buildings they placed. Adding deconstruct completes the build/remove loop and makes colony layout editable without restarting.

## What Changes

- Add a **Deconstruct** toolbar tool (mutually exclusive with Select and build modes)
- Handle `IncomingEvent::Deconstruct { x, y }` in the simulation
- **Instant cancel** for construction sites at 0% progress (no colonist labor)
- **Labor-based removal** for all other targets via a new `DeconstructionSite` component and `TaskKind::Deconstruct`
- Add `work_to_deconstruct` per building in `content/base/buildings.yaml` (parsed by `ContentRegistry`)
- Allow deconstruct orders while a target is in use; colonists assign only when the target is free
- Idle colonists pick the **nearest** available Build or Deconstruct job (no fixed priority)
- Expose `deconstruction_sites` in state snapshots for client rendering (red overlay + progress bar)
- Persist deconstruction state through save/load

## Capabilities

### New Capabilities

_(none — deconstruct extends existing simulation, view, and content capabilities)_

### Modified Capabilities

- `world-simulation`: deconstruct command handling, instant 0% cancel, deconstruction completion removes buildings
- `colonist-simulation`: `TaskKind::Deconstruct`, assignment pool with Build, availability gate, execution, preemption
- `view-layer`: Deconstruct toolbar button, click routing, red deconstruction overlay rendering
- `worker-bridge`: new `deconstruct` event and `deconstruction_sites` snapshot field
- `content-definitions`: `work_to_deconstruct` field on building definitions
- `game-state-persistence`: save/load includes pending deconstruction sites

## Impact

- **Content**: `content/base/buildings.yaml` — add `work_to_deconstruct` to wall, bed, berry_bush
- **game-core (Rust)**: `DeconstructionSite` component, `TaskKind::Deconstruct`, event/snapshot types, assignment and execution systems, save/load
- **client (TypeScript/Vue)**: toolbar, `ToolMode`, click handler, PixiJS rendering, protocol types
- **Tests**: game-core unit tests for instant cancel, labor path, availability gate, nearest-job selection, preemption, snapshot round-trip
