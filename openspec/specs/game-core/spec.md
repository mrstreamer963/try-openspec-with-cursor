# Game Core

## Purpose

Rust WASM game core using bevy_ecs: tick loop, event protocol, and entity management.

## Requirements

### Requirement: Content pack initialization
The game core SHALL accept a content pack payload at `Game` construction (or via a dedicated init path before the first tick) and build a validated `ContentRegistry` from it.

#### Scenario: Game starts with bundled base pack
- **WHEN** the WASM `Game` constructor receives the base YAML content payload from the host
- **THEN** simulation uses definitions from the registry instead of hardcoded enums

#### Scenario: Init failure surfaces error
- **WHEN** content validation fails during `Game` construction
- **THEN** the host receives an error and no partially initialized game instance is used for ticks

### Requirement: Content registry ECS resource
The game core SHALL store loaded content definitions as a bevy_ecs `Resource` accessible to all simulation systems.

#### Scenario: Systems read building defs from resource
- **WHEN** construction completes for a building id
- **THEN** spawned entity components are derived from that building's YAML `on_complete` primitives via the registry

### Requirement: Manual tick loop
The game core SHALL expose a manual `tick(dt: f32)` function that advances all ECS systems by the given delta time in seconds.

#### Scenario: Single tick advances simulation
- **WHEN** the host calls `tick(0.05)` once
- **THEN** all registered ECS systems execute exactly one update cycle with dt = 0.05

#### Scenario: Zero delta time is a no-op
- **WHEN** the host calls `tick(0.0)`
- **THEN** no simulation state changes occur

### Requirement: Typed event protocol
The game core SHALL define `IncomingEvent` and `OutgoingEvent` enums serialized via serde and exposed through wasm-bindgen for communication with the host. Building, terrain, and task references in snapshots and build commands SHALL use string content IDs defined in the active content pack.

#### Scenario: Host sends pause command
- **WHEN** the host dispatches an `IncomingEvent::SetPaused(true)` event
- **THEN** the simulation stops advancing on subsequent ticks until unpaused

#### Scenario: Core emits state snapshot
- **WHEN** a tick completes and state has changed
- **THEN** the core produces an `OutgoingEvent::StateSnapshot` containing serializable world state with content id strings for terrain and buildings

#### Scenario: Build command uses content id
- **WHEN** the host dispatches `IncomingEvent::Build` with building id `berry_bush`
- **THEN** the core resolves the id via the content registry and creates a construction order if the id is buildable

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
The game core SHALL accept an `IncomingEvent::LoadState` carrying a `StateSnapshot` and rebuild the ECS world and `WorldGrid` to match that snapshot. Snapshot building and terrain fields SHALL use string content IDs that exist in the loaded content registry.

#### Scenario: Restore terrain and buildings
- **WHEN** `LoadState` is dispatched with a snapshot containing tiles and buildings using content id strings
- **THEN** `WorldGrid` terrain and building layers match the snapshot and building entities exist with correct types and supply components per building definitions

#### Scenario: Restore colonists
- **WHEN** `LoadState` includes colonist entries with ids, names, positions, needs, active status flags, and task kinds
- **THEN** colonist entities are spawned with matching components derived from the content registry

#### Scenario: Restore construction sites
- **WHEN** `LoadState` includes construction site entries with building content ids and progress values
- **THEN** construction site entities exist at the given coordinates with `work_remaining` consistent with the saved progress and building definition

#### Scenario: Restore simulation controls
- **WHEN** `LoadState` includes `paused` and `speed` fields
- **THEN** the game core applies those values so subsequent ticks and snapshots reflect the restored pause and speed settings

#### Scenario: Invalid load state rejected
- **WHEN** `LoadState` carries a snapshot that fails core validation (e.g. wrong tile count or unknown content id)
- **THEN** the core returns `OutgoingEvent::Error` and leaves the previous simulation state intact

### Requirement: Snapshot export for persistence
The game core SHALL produce `StateSnapshot` payloads suitable for the versioned save file `state` field via the existing `get_snapshot` / tick snapshot path without additional transformation.

#### Scenario: Snapshot matches save state field
- **WHEN** the host calls `get_snapshot()` after simulation changes
- **THEN** the returned JSON deserializes to a `StateSnapshot` that can be embedded in a version-1 save file
