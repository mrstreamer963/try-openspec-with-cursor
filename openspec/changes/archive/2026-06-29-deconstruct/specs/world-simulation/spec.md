## ADDED Requirements

### Requirement: Deconstruct command handling
When the host sends a deconstruct command for a tile, the simulation SHALL apply removal rules based on tile state. Empty tiles and tiles with an existing `DeconstructionSite` SHALL reject the command silently.

#### Scenario: Reject deconstruct on empty tile
- **WHEN** the host sends a deconstruct command at a tile with no construction site or finished building
- **THEN** the command is rejected silently and no entity is created or removed

#### Scenario: Reject duplicate deconstruct order
- **WHEN** the host sends a deconstruct command at a tile that already has a pending `DeconstructionSite`
- **THEN** the command is rejected and the existing order is unchanged

#### Scenario: Instant cancel at zero progress
- **WHEN** the host sends a deconstruct command at a `ConstructionSite` with `progress == 0` (i.e. `work_remaining == work_required`)
- **THEN** the construction site is despawned immediately, any builder `reserved_by` is released, and no `DeconstructionSite` is spawned

#### Scenario: Deconstruct in-progress construction site
- **WHEN** the host sends a deconstruct command at a `ConstructionSite` with `progress > 0`
- **THEN** the builder reservation is released, the `ConstructionSite` is despawned, and a `DeconstructionSite` is spawned with `work_remaining = work_to_deconstruct` for that building type

#### Scenario: Deconstruct finished building
- **WHEN** the host sends a deconstruct command at a tile with a finished building and no pending deconstruction
- **THEN** a `DeconstructionSite` is spawned with `work_remaining = work_to_deconstruct` for that building type and the building remains on the grid until deconstruction completes

### Requirement: Deconstruction completion
When a `DeconstructionSite`'s remaining work reaches zero, the simulation SHALL remove the target building from the world grid (if present), despawn the building ECS entity, despawn the `DeconstructionSite`, and clear the assigned colonist's task to Idle.

#### Scenario: Complete wall deconstruction
- **WHEN** a `wall` deconstruction site's work reaches zero while a colonist is adjacent
- **THEN** the wall is removed from the world grid, the building entity is despawned, the deconstruction site is removed, and the colonist's task is cleared to Idle

#### Scenario: Target removed before completion
- **WHEN** a `DeconstructionSite` exists but its target building or site is removed by another system (e.g. berry bush depleted)
- **THEN** the deconstruction site is despawned and any assigned colonist's task and reservation are cleared
