## ADDED Requirements

### Requirement: Deconstruct command forwarding
The worker bridge SHALL forward deconstruct commands from the main thread to the WASM game core as `IncomingEvent::Deconstruct { x, y }`.

#### Scenario: Deconstruct event forwarded
- **WHEN** the main thread sends a deconstruct command with tile coordinates `(x, y)`
- **THEN** the worker deserializes and dispatches `IncomingEvent::Deconstruct { x, y }` to the game core

### Requirement: Deconstruction sites in snapshot
The worker bridge SHALL include `deconstruction_sites` from the game core snapshot when posting state updates to the main thread.

#### Scenario: Snapshot includes deconstruction sites
- **WHEN** the game core emits a state snapshot with pending deconstruction sites
- **THEN** the worker posts the snapshot including a `deconstruction_sites` array with x, y, building id, and progress for each site
