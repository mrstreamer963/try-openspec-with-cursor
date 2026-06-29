# World Simulation

## Purpose

50×50 tile world generation, terrain types, and placeable buildings.

## Requirements

### Requirement: 50×50 tile world
The simulation SHALL generate a fixed 50×50 grid world at initialization.

#### Scenario: World dimensions
- **WHEN** the game starts
- **THEN** the world contains exactly 2500 tiles arranged in a 50-column by 50-row grid

#### Scenario: Deterministic generation
- **WHEN** the game is initialized with the same seed twice
- **THEN** both worlds produce identical tile layouts

### Requirement: Terrain types
Each tile SHALL have a terrain type identified by a content id from the loaded terrain definitions. The base content pack SHALL define `water`, `sand`, and `grass`. Walkability SHALL be determined by each terrain definition's `walkable` field.

#### Scenario: Terrain assignment
- **WHEN** the world is generated
- **THEN** every tile is assigned exactly one terrain content id from the loaded terrain definitions

#### Scenario: Walkability by terrain
- **WHEN** a colonist attempts to pathfind through a tile whose terrain definition has `walkable: false`
- **THEN** that tile is treated as impassable

### Requirement: Placeable buildings
The simulation SHALL support placing buildings defined in the loaded content pack with `buildable: true`. The base content pack SHALL include `bed`, `berry_bush`, and `wall`. Build commands from the host SHALL reference building content ids.

#### Scenario: Place bed on grass
- **WHEN** the host sends a build command for `bed` at a walkable grass tile
- **THEN** a construction order for `bed` is created at that tile and, once complete, a bed building entity exists at that tile with components specified by the bed definition

#### Scenario: Reject build on water
- **WHEN** the host sends a build command at a water tile
- **THEN** the build is rejected and no construction order or building entity is created

#### Scenario: Berry bush provides food
- **WHEN** a colonist interacts with a completed `berry_bush` from an orthogonally adjacent tile and the bush has berries remaining per its supply primitive
- **THEN** the colonist's food need increases and the bush loses one berry

#### Scenario: Depleted berry bush removed
- **WHEN** a `berry_bush`'s last berry is consumed
- **THEN** the building is removed from the world grid and no longer appears in snapshots

#### Scenario: Bed satisfies sleep
- **WHEN** a colonist interacts with a completed `bed` while standing on the bed tile and holding the bed reservation
- **THEN** the colonist's sleep need increases per the bed interaction definition

### Requirement: Resource-free construction
Building placement SHALL NOT require any resource cost in v1.

#### Scenario: Unlimited building
- **WHEN** the player places multiple beds in succession
- **THEN** each construction order is created without checking or deducting resources

### Requirement: Finite berry supply
Each newly completed `berry_bush` SHALL start with berry supply equal to the amount configured in the building definition's `on_complete` supply primitive. The base pack value SHALL be 3.

#### Scenario: New bush berry count
- **WHEN** a `berry_bush` construction order completes
- **THEN** the finished bush has exactly the configured starting berry count from YAML

### Requirement: Construction orders from build commands
When the host sends a build command for a valid tile, the simulation SHALL create a construction order at that tile instead of an instant finished building.

#### Scenario: Valid construction order
- **WHEN** the host sends a build command for a Wall at a walkable empty tile with no pending construction
- **THEN** a construction order for a Wall is created at that tile

#### Scenario: Reject duplicate construction order
- **WHEN** the host sends a build command at a tile that already has a pending construction order
- **THEN** the command is rejected and the existing order is unchanged

#### Scenario: Reject build on occupied tile
- **WHEN** the host sends a build command at a tile that already has a finished building
- **THEN** the command is rejected

### Requirement: Construction completion
When a construction order's remaining work reaches zero, the simulation SHALL replace it with a finished building on the world grid using the building definition's `work_required` and `on_complete` primitives.

#### Scenario: Complete wall construction
- **WHEN** a `wall` construction order's work reaches zero
- **THEN** a wall building entity exists at that tile, the construction order is removed, and the tile blocks movement per the wall definition

#### Scenario: Construction site walkable until complete
- **WHEN** a construction order exists at a tile and work remains
- **THEN** the tile is walkable and no finished building occupies the world grid at that cell

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
