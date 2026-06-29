## ADDED Requirements

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
