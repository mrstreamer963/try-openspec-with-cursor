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

### Requirement: Unique settled cell occupancy
At most one colonist SHALL occupy a grid cell when movement completes a step to that cell. A colonist's settled cell is its current grid cell derived from position.

#### Scenario: Second colonist waits for occupied cell
- **WHEN** a colonist completes movement toward a waypoint cell that is already occupied by another colonist's settled position
- **THEN** the colonist does not snap to that cell, does not advance its path index, and remains at its previous position until the cell is free

#### Scenario: Idle colonist occupies its cell
- **WHEN** a colonist is idle (including after completing an Eat task on a stand tile)
- **THEN** its current grid cell is occupied and no other colonist may settle on that cell

### Requirement: Pass-through during movement
Colonists MAY pass through each other while moving between grid cells. Pathfinding SHALL NOT treat other colonists as impassable obstacles.

#### Scenario: Colonists cross paths mid-movement
- **WHEN** two colonists move toward each other and their interpolated positions overlap before either completes a waypoint snap
- **THEN** both continue moving without blocking each other

#### Scenario: Pathfinding ignores colonist positions
- **WHEN** A* pathfinding computes a route
- **THEN** only terrain and building walkability affect the path, not colonist positions

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need. Pathfinding SHALL use the colonist's current grid cell, derived by flooring world coordinates. When multiple needs are below the threshold, the simulation SHALL try needs in priority order (Food, then Sleep) and assign the first need for which a satisfiable target and valid path exist.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a completed BerryBush with berries remaining exists with at least one adjacent walkable stand tile
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush, with path destination on an adjacent stand tile

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist's Sleep need drops below the threshold and an unoccupied Bed exists
- **THEN** the colonist is assigned a Sleep task targeting the nearest such Bed and the bed is reserved for that colonist

#### Scenario: Food priority when both needs critical
- **WHEN** a colonist's Food and Sleep needs are both below the threshold and both a satisfiable BerryBush and an unoccupied Bed exist
- **THEN** the colonist is assigned an Eat task (Food takes priority over Sleep)

#### Scenario: Fallback to sleep when food unavailable
- **WHEN** a colonist's Food and Sleep needs are both below the threshold, no satisfiable BerryBush exists, and an unoccupied Bed exists with a valid path
- **THEN** the colonist is assigned a Sleep task targeting the nearest such Bed and the bed is reserved for that colonist

#### Scenario: Fallback to eat when sleep unavailable
- **WHEN** a colonist's Food and Sleep needs are both below the threshold, no satisfiable Bed exists, and a BerryBush with berries and a valid eat stand path exists
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush

#### Scenario: No need task when nothing satisfiable
- **WHEN** a colonist has one or more needs below the threshold but no satisfiable target and valid path exist for any critical need
- **THEN** the colonist remains idle with no Eat or Sleep task assigned

#### Scenario: Need assignment replaces wander path
- **WHEN** an idle colonist is following a wander path and a critical need becomes assignable (including via fallback)
- **THEN** the wander path is replaced by the need-satisfying task path on the same assignment pass

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold and no construction orders are unassigned
- **THEN** the colonist remains idle and the simulation assigns idle wander movement (see Idle wander requirement)

### Requirement: Idle wander
When a colonist is idle with no Eat, Sleep, or Build assignment and no active path, the simulation SHALL assign a path to a random nearby walkable cell within a configurable wander radius.

#### Scenario: Wander when fully idle
- **WHEN** a colonist is idle, all needs are above threshold, no unassigned construction orders exist, and the colonist has no remaining path waypoints
- **THEN** the colonist is assigned a path to a random walkable cell within the wander radius

#### Scenario: New wander target on arrival
- **WHEN** an idle colonist completes its wander path and remains idle
- **THEN** on the next assignment pass the colonist is assigned a new random nearby wander destination

#### Scenario: Wander preempted by need
- **WHEN** an idle colonist is wandering and Food or Sleep drops below the critical threshold
- **THEN** the wander path is replaced by a need-satisfying task assignment

#### Scenario: Wander preempted by build
- **WHEN** an idle colonist is wandering and an unassigned construction order becomes available
- **THEN** the wander path is replaced by a Build task assignment

#### Scenario: Wander excludes current and occupied cells
- **WHEN** selecting a wander destination
- **THEN** the simulation excludes the colonist's current grid cell and cells occupied by other colonists' settled positions from candidate targets

#### Scenario: Wander uses pathfinding
- **WHEN** a wander destination is selected
- **THEN** the colonist follows a valid A* path to that cell; if no path exists after retry attempts, the colonist remains idle until the next assignment pass

#### Scenario: Task stays Idle during wander
- **WHEN** a colonist is following a wander path
- **THEN** the colonist's task kind remains Idle in the state snapshot

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
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles. Other colonists SHALL NOT be treated as impassable during path computation.

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles but not avoiding cells occupied by other colonists

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

### Requirement: Eat assignment prefers unoccupied stand tiles
When assigning an Eat task, the simulation SHALL select an adjacent stand tile that is not occupied by another colonist's settled position and not already assigned as a stand to another colonist in the same assignment pass.

#### Scenario: Occupied stand skipped for eat assignment
- **WHEN** a colonist needs Food and the nearest BerryBush has adjacent stand tiles, but the best stand tile is occupied by another colonist
- **THEN** the simulation assigns an Eat task using another free adjacent stand tile for that bush, or a stand tile for another bush, or no Eat task if none are available

#### Scenario: Single stand queue at bush
- **WHEN** two colonists need Food and a BerryBush has only one adjacent stand tile
- **THEN** at most one colonist is assigned that stand tile per assignment pass; the other colonist is assigned elsewhere or remains idle until the stand is free

### Requirement: Single bed occupancy
At most one colonist SHALL occupy or be reserved for a given Bed at any time.

#### Scenario: Second colonist skips occupied bed
- **WHEN** a colonist needs Sleep and the nearest Bed is already reserved by another colonist
- **THEN** the colonist is assigned a Sleep task to the nearest unoccupied Bed, or remains idle if none exist

#### Scenario: Bed released after sleep
- **WHEN** a colonist completes or fails a Sleep task that held a bed reservation
- **THEN** the bed becomes available for other colonists
