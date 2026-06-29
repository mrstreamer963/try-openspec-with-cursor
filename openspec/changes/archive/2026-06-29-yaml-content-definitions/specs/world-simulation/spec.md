## MODIFIED Requirements

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

### Requirement: Finite berry supply
Each newly completed `berry_bush` SHALL start with berry supply equal to the amount configured in the building definition's `on_complete` supply primitive. The base pack value SHALL be 3.

#### Scenario: New bush berry count
- **WHEN** a `berry_bush` construction order completes
- **THEN** the finished bush has exactly the configured starting berry count from YAML

### Requirement: Construction completion
When a construction order's remaining work reaches zero, the simulation SHALL replace it with a finished building on the world grid using the building definition's `work_required` and `on_complete` primitives.

#### Scenario: Complete wall construction
- **WHEN** a `wall` construction order's work reaches zero
- **THEN** a wall building entity exists at that tile, the construction order is removed, and the tile blocks movement per the wall definition

#### Scenario: Construction site walkable until complete
- **WHEN** a construction order exists at a tile and work remains
- **THEN** the tile is walkable and no finished building occupies the world grid at that cell
