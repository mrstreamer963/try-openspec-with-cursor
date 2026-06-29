# Game Core

## Purpose

Rust WASM game core using bevy_ecs: tick loop, event protocol, and entity management.

## Requirements

### Requirement: Manual tick loop
The game core SHALL expose a manual `tick(dt: f32)` function that advances all ECS systems by the given delta time in seconds.

#### Scenario: Single tick advances simulation
- **WHEN** the host calls `tick(0.05)` once
- **THEN** all registered ECS systems execute exactly one update cycle with dt = 0.05

#### Scenario: Zero delta time is a no-op
- **WHEN** the host calls `tick(0.0)`
- **THEN** no simulation state changes occur

### Requirement: Typed event protocol
The game core SHALL define `IncomingEvent` and `OutgoingEvent` enums serialized via serde and exposed through wasm-bindgen for communication with the host.

#### Scenario: Host sends pause command
- **WHEN** the host dispatches an `IncomingEvent::SetPaused(true)` event
- **THEN** the simulation stops advancing on subsequent ticks until unpaused

#### Scenario: Core emits state snapshot
- **WHEN** a tick completes and state has changed
- **THEN** the core produces an `OutgoingEvent::StateSnapshot` containing serializable world state

### Requirement: ECS entity management
The game core SHALL use bevy_ecs to manage all simulation entities (tiles, colonists, buildings) as components on entities.

#### Scenario: Entity creation
- **WHEN** a new colonist is spawned during initialization
- **THEN** a bevy_ecs entity is created with Position, Needs, and Task components attached

#### Scenario: Entity removal
- **WHEN** an entity is despawned
- **THEN** all associated components are removed and the entity ID is no longer referenced

### Requirement: WASM compilation target
The game core SHALL compile to a WebAssembly module loadable in a browser WebWorker via wasm-bindgen.

#### Scenario: Module initialization
- **WHEN** the WASM module is instantiated in a WebWorker
- **THEN** a `Game` instance is created and ready to receive events and tick calls

### Requirement: Load state from snapshot
The game core SHALL accept an `IncomingEvent::LoadState` carrying a `StateSnapshot` and rebuild the ECS world and `WorldGrid` to match that snapshot.

#### Scenario: Restore terrain and buildings
- **WHEN** `LoadState` is dispatched with a snapshot containing tiles and buildings
- **THEN** `WorldGrid` terrain and building layers match the snapshot and building entities exist with correct types and berry supply where applicable

#### Scenario: Restore colonists
- **WHEN** `LoadState` includes colonist entries with ids, names, positions, needs, and task kinds
- **THEN** colonist entities are spawned with matching `ColonistId`, `ColonistName`, `Position`, `Needs`, need buff markers, and `Task.kind`

#### Scenario: Restore construction sites
- **WHEN** `LoadState` includes construction site entries with progress values
- **THEN** construction site entities exist at the given coordinates with `work_remaining` consistent with the saved progress

#### Scenario: Restore simulation controls
- **WHEN** `LoadState` includes `paused` and `speed` fields
- **THEN** the game core applies those values so subsequent ticks and snapshots reflect the restored pause and speed settings

#### Scenario: Invalid load state rejected
- **WHEN** `LoadState` carries a snapshot that fails core validation (e.g. wrong tile count)
- **THEN** the core returns `OutgoingEvent::Error` and leaves the previous simulation state intact

### Requirement: Snapshot export for persistence
The game core SHALL produce `StateSnapshot` payloads suitable for the versioned save file `state` field via the existing `get_snapshot` / tick snapshot path without additional transformation.

#### Scenario: Snapshot matches save state field
- **WHEN** the host calls `get_snapshot()` after simulation changes
- **THEN** the returned JSON deserializes to a `StateSnapshot` that can be embedded in a version-1 save file
