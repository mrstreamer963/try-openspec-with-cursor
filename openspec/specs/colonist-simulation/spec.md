# Colonist Simulation

## Purpose

Simulates colonist entities: needs, automatic task assignment, pathfinding, and task execution.

## Requirements

### Requirement: Three starting colonists
The simulation SHALL spawn exactly 3 colonists at game start on walkable tiles.

#### Scenario: Initial colonist count
- **WHEN** the game initializes
- **THEN** exactly 3 colonist entities exist with unique IDs, unique display names, and valid positions

### Requirement: Colonist display names
Each colonist SHALL have a unique display name assigned at spawn from a fixed name pool.

#### Scenario: Unique names at spawn
- **WHEN** the game initializes with 3 colonists
- **THEN** each colonist has a distinct non-empty name

#### Scenario: Name in snapshot
- **WHEN** a state snapshot is built
- **THEN** each colonist entry includes its display name alongside its numeric id

### Requirement: Colonist needs
Each colonist SHALL have two needs: Food and Sleep, each represented as a value from 0 (critical) to 100 (satisfied).

#### Scenario: Needs decay over time
- **WHEN** the simulation ticks while a colonist is idle
- **THEN** both Food and Sleep values decrease at a configurable rate

#### Scenario: Critical need threshold
- **WHEN** a colonist's Food or Sleep value drops below a defined threshold (e.g., 30)
- **THEN** the colonist is flagged as needing that resource

### Requirement: Float world position
Each colonist SHALL have a world position expressed as floating-point coordinates in tile units, where integer values align to tile centers (e.g. cell `(5, 7)` is world position `(5.0, 7.0)`).

#### Scenario: Spawn at tile center
- **WHEN** a colonist is spawned on grid cell `(x, y)`
- **THEN** its world position is initialized to `(x as f32, y as f32)`

#### Scenario: Snapshot exposes float position
- **WHEN** a state snapshot is built
- **THEN** each colonist's `x` and `y` fields are floating-point tile coordinates

### Requirement: Continuous movement
Colonists SHALL move continuously between path waypoints at a configurable speed (`MOVE_SPEED` tiles per second), advancing by fractional tile amounts each tick without rounding position to integer cells during transit.

#### Scenario: Sub-tile movement per tick
- **WHEN** a colonist is moving toward a waypoint and the remaining distance exceeds one tick's travel distance
- **THEN** the colonist's world position advances by a fractional amount less than one full tile

#### Scenario: Arrive at waypoint
- **WHEN** a colonist's remaining distance to the current waypoint is less than or equal to one tick's travel distance
- **THEN** the colonist snaps to the waypoint coordinates and advances to the next waypoint

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need. Pathfinding SHALL use the colonist's current grid cell, derived by flooring world coordinates.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a completed BerryBush with berries remaining exists with at least one adjacent walkable stand tile
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush, with path destination on an adjacent stand tile

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist's Sleep need drops below the threshold and an unoccupied Bed exists
- **THEN** the colonist is assigned a Sleep task targeting the nearest such Bed and the bed is reserved for that colonist

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold and no construction orders are unassigned
- **THEN** the colonist remains idle with no assigned task

### Requirement: Build task assignment
When a colonist has no critical need and a construction order exists without an assigned builder, the simulation SHALL assign a Build task to the nearest such order.

#### Scenario: Auto-assign build task
- **WHEN** a colonist is idle, all needs are above threshold, and an unassigned construction order exists
- **THEN** the colonist is assigned a Build task targeting that construction site

#### Scenario: Needs override build
- **WHEN** a colonist is building and Food or Sleep drops below the critical threshold
- **THEN** the Build task is cleared, the construction site reservation is released, and a need-satisfying task is assigned instead

#### Scenario: One builder per construction site
- **WHEN** a construction order is already reserved by another colonist
- **THEN** other idle colonists skip that order and target the next nearest unassigned site

### Requirement: Build task execution
When a colonist with a Build task is on the construction site tile, the simulation SHALL apply construction work each tick until the order completes or the task is cleared.

#### Scenario: Work at construction site
- **WHEN** a colonist with a Build task arrives at the construction site tile
- **THEN** construction work progress increases each simulation tick

#### Scenario: Complete build task
- **WHEN** construction work on the assigned site reaches completion while the colonist is present
- **THEN** the finished building is created, the construction order is removed, and the colonist's task is cleared to Idle

#### Scenario: Path to construction site
- **WHEN** a colonist is assigned a Build task at a distant construction site
- **THEN** the colonist follows a valid A* path to the site tile

#### Scenario: No path to construction site
- **WHEN** no walkable path exists to the construction site
- **THEN** the Build task is cancelled, the site reservation is released, and the colonist returns to idle

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles.

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles

#### Scenario: No path available
- **WHEN** no walkable path exists to the task stand tile
- **THEN** the task is cancelled and the colonist returns to idle

### Requirement: Task execution
When a colonist reaches its task destination, the simulation SHALL execute the task interaction and restore the relevant need. Arrival SHALL be determined when the path is complete and the colonist's floored grid cell matches the task target cell.

#### Scenario: Complete eat task
- **WHEN** a colonist with an Eat task arrives at its stand tile orthogonally adjacent to a BerryBush with berries remaining
- **THEN** the colonist's Food need is restored, one berry is consumed, and the task is cleared

#### Scenario: Complete sleep task
- **WHEN** a colonist with a Sleep task arrives at a Bed tile reserved for that colonist
- **THEN** the colonist's Sleep need is restored, the bed reservation is released, and the task is cleared

### Requirement: Eat fails on depleted bush
When a colonist completes an Eat task at a stand tile but the adjacent bush has no berries, the simulation SHALL not restore Food and SHALL clear the task.

#### Scenario: Bush depleted before arrival
- **WHEN** a colonist arrives at its Eat stand tile but the adjacent BerryBush has no berries (or no bush)
- **THEN** the colonist's Food need is unchanged and the task is cleared

### Requirement: No eat task without stand tile
When no orthogonally adjacent walkable tile exists next to a BerryBush, the simulation SHALL not assign an Eat task to that bush.

#### Scenario: Bush surrounded by obstacles
- **WHEN** a colonist needs Food and the only BerryBush with berries has no adjacent walkable tiles
- **THEN** no Eat task is assigned for that bush and the colonist remains idle or seeks another bush

### Requirement: Single bed occupancy
At most one colonist SHALL occupy or be reserved for a given Bed at any time.

#### Scenario: Second colonist skips occupied bed
- **WHEN** a colonist needs Sleep and the nearest Bed is already reserved by another colonist
- **THEN** the colonist is assigned a Sleep task to the nearest unoccupied Bed, or remains idle if none exist

#### Scenario: Bed released after sleep
- **WHEN** a colonist completes or fails a Sleep task that held a bed reservation
- **THEN** the bed becomes available for other colonists
