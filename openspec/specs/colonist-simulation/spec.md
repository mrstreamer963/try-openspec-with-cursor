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
Each colonist SHALL have one floating-point value per need defined in the loaded content pack. The base content pack SHALL define `food` and `sleep`, each from 0 (critical) to the need's configured `max` (default 100).

#### Scenario: Needs decay over time
- **WHEN** the simulation ticks while a colonist is idle
- **THEN** each need value decreases at the `decay_per_sec` rate from its YAML definition

#### Scenario: Critical need threshold
- **WHEN** a colonist's need value drops below that need's `critical_threshold` from YAML
- **THEN** the colonist receives the status(es) whose `apply_when` condition matches that need

#### Scenario: Need status in snapshot
- **WHEN** a state snapshot is built
- **THEN** each colonist entry includes `hungry: true` when the `hungry` status is active and `wants_sleep: true` when the `wants_sleep` status is active; otherwise the corresponding flag is `false`

### Requirement: Float world position
Each colonist SHALL have a world position expressed as floating-point coordinates in tile units, where integer values align to tile centers (e.g. cell `(5, 7)` is world position `(5.0, 7.0)`).

#### Scenario: Spawn at tile center
- **WHEN** a colonist is spawned on grid cell `(x, y)`
- **THEN** its world position is initialized to `(x as f32, y as f32)`

#### Scenario: Snapshot exposes float position
- **WHEN** a state snapshot is built
- **THEN** each colonist's `x` and `y` fields are floating-point tile coordinates

#### Scenario: At-task-stand flag in snapshot
- **WHEN** a state snapshot is built and a colonist is locked at its task stand (including a build worker adjacent to a construction site)
- **THEN** the colonist entry includes `at_task_stand: true`; otherwise `at_task_stand` is `false`

### Requirement: Continuous movement
Colonists SHALL move continuously between path waypoints at a configurable speed (`MOVE_SPEED` tiles per second), advancing by fractional tile amounts each tick without rounding position to integer cells during transit.

#### Scenario: Sub-tile movement per tick
- **WHEN** a colonist is moving toward a waypoint and the remaining distance exceeds one tick's travel distance
- **THEN** the colonist's world position advances by a fractional amount less than one full tile

#### Scenario: Arrive at waypoint
- **WHEN** a colonist's remaining distance to the current waypoint is less than or equal to one tick's travel distance
- **THEN** the colonist snaps to the waypoint coordinates and advances to the next waypoint

#### Scenario: High-speed movement substepping
- **WHEN** a simulation tick uses a scaled `dt` greater than the movement substep interval (e.g. at 5× or 10× game speed)
- **THEN** colonist movement is applied in fixed substeps so per-tick travel distance matches 1× simulation

### Requirement: Unique settled cell occupancy
At most one colonist SHALL occupy a grid cell when movement completes a step to that cell. A colonist's settled cell is its current grid cell derived from position.

#### Scenario: Second colonist waits for occupied cell
- **WHEN** a colonist completes movement toward a waypoint cell that is already occupied by another colonist's settled position
- **THEN** the colonist does not snap to that cell, does not advance its path index, does not partial-step toward that cell, and remains at its previous position until the cell is free

#### Scenario: No drift into occupied waypoint
- **WHEN** a colonist is moving toward a waypoint cell that is occupied by another colonist
- **THEN** the colonist holds position rather than advancing fractionally toward the blocked cell

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
When a colonist has an active status that assigns task priority via YAML effects, the simulation SHALL automatically assign tasks to satisfy the linked need. Pathfinding SHALL use the colonist's current grid cell, derived by flooring world coordinates. When multiple critical statuses apply, the simulation SHALL use status effect priorities from the content pack (base pack: food before sleep) and assign the first satisfiable task.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist has the `hungry` status and a completed `berry_bush` with berries exists with at least one adjacent walkable stand tile
- **THEN** the colonist is assigned an Eat task targeting the nearest such bush per the berry bush interaction definition

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist has the `wants_sleep` status and an unoccupied `bed` exists
- **THEN** the colonist is assigned a Sleep task with path destination on the bed tile, and the bed is reserved for that colonist

#### Scenario: Food priority when both needs critical
- **WHEN** a colonist has both `hungry` and `wants_sleep` active and both a satisfiable `berry_bush` and an unoccupied `bed` exist
- **THEN** the colonist is assigned an Eat task (food priority per base pack status effect ordering)

#### Scenario: Fallback to sleep when food unavailable
- **WHEN** a colonist has both critical statuses, no satisfiable `berry_bush` exists, and an unoccupied `bed` exists with a valid path
- **THEN** the colonist is assigned a Sleep task targeting the nearest such bed

#### Scenario: Fallback to eat when sleep unavailable
- **WHEN** a colonist has both critical statuses, no satisfiable `bed` exists, and a `berry_bush` with berries and a valid eat stand path exists
- **THEN** the colonist is assigned an Eat task targeting the nearest such bush

#### Scenario: No need task when nothing satisfiable
- **WHEN** a colonist has one or more active critical statuses but no satisfiable target and valid path exist
- **THEN** the colonist remains idle with no Eat or Sleep task assigned and the simulation assigns idle wander movement if the colonist has no remaining path waypoints

#### Scenario: Need assignment replaces wander path
- **WHEN** an idle colonist is following a wander path and a critical need becomes assignable (including via fallback)
- **THEN** the wander path is replaced by the need-satisfying task path on the same assignment pass

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold and no construction orders are unassigned
- **THEN** the colonist remains idle and the simulation assigns idle wander movement (see Idle wander requirement)

### Requirement: Idle wander
When a colonist is idle with no Eat, Sleep, Build, or Deconstruct assignment and no active path, the simulation SHALL assign a path to a random nearby walkable cell within a configurable wander radius. This includes colonists whose critical needs cannot be satisfied on the current assignment pass.

#### Scenario: Wander when fully idle
- **WHEN** a colonist is idle, all needs are above threshold, no unassigned construction or deconstruction orders exist, and the colonist has no remaining path waypoints
- **THEN** the colonist is assigned a path to a random walkable cell within the wander radius

#### Scenario: Wander when critical needs unsatisfiable
- **WHEN** a colonist is idle, one or more needs are below the critical threshold, no Eat or Sleep task can be assigned, and the colonist has no remaining path waypoints
- **THEN** the colonist is assigned a wander path while remaining task kind Idle

#### Scenario: New wander target on arrival
- **WHEN** an idle colonist completes its wander path and remains idle
- **THEN** on the next assignment pass the colonist is assigned a new random nearby wander destination

#### Scenario: Wander preempted by need
- **WHEN** an idle colonist is wandering and Food or Sleep drops below the critical threshold
- **THEN** the wander path is replaced by a need-satisfying task assignment

#### Scenario: Wander preempted by build or deconstruct
- **WHEN** an idle colonist is wandering and an unassigned construction or deconstruction order becomes available
- **THEN** the wander path is replaced by a Build or Deconstruct task assignment

#### Scenario: Wander excludes current and occupied cells
- **WHEN** selecting a wander destination
- **THEN** the simulation excludes the colonist's current grid cell and cells occupied by other colonists' settled positions from candidate targets

#### Scenario: Wander avoids bush destination
- **WHEN** selecting a wander destination
- **THEN** the simulation does not select a tile occupied by a building with `blocks_settle: true`

#### Scenario: Wander uses pathfinding
- **WHEN** a wander destination is selected
- **THEN** the colonist follows a valid A* path to that cell; if no path exists after retry attempts, the colonist remains idle until the next assignment pass

#### Scenario: Task stays Idle during wander
- **WHEN** a colonist is following a wander path
- **THEN** the colonist's task kind remains Idle in the state snapshot

### Requirement: Build task assignment
When a colonist has no critical need, the simulation SHALL assign Build or Deconstruct tasks from a merged pool of unreserved construction and deconstruction sites. The colonist SHALL receive the nearest reachable candidate (Manhattan distance to stand tile). Deconstruction sites SHALL be included only when they pass the availability gate (see Deconstruct target availability requirement).

#### Scenario: Auto-assign build task
- **WHEN** a colonist is idle, all needs are above threshold, and an unassigned construction order exists
- **THEN** the colonist is assigned a Build task targeting that construction site

#### Scenario: Needs override build and deconstruct
- **WHEN** a colonist is building or deconstructing and Food or Sleep drops below the critical threshold
- **THEN** the active task is cleared, the site reservation is released, and a need-satisfying task is assigned instead

#### Scenario: One worker per site
- **WHEN** a construction or deconstruction site is already reserved by another colonist
- **THEN** other idle colonists skip that site and target the next nearest unassigned site

#### Scenario: Nearest job wins over type
- **WHEN** an idle colonist has both an unassigned construction site and an assignable deconstruction site within reach
- **THEN** the colonist is assigned whichever site has the nearest reachable stand tile, regardless of Build vs Deconstruct

### Requirement: Build task execution
When a colonist with a Build task is orthogonally adjacent to the construction site on a valid stand tile, the simulation SHALL apply construction work each tick until the order completes or the task is cleared.

#### Scenario: Work at adjacent stand
- **WHEN** a colonist with a Build task is orthogonally adjacent to the construction site with no remaining path waypoints
- **THEN** construction work progress increases each simulation tick

#### Scenario: Builder locked in place when adjacent
- **WHEN** a colonist with a Build task reaches any orthogonally adjacent cell to the construction site
- **THEN** the colonist stops moving, its path is cleared, and it builds from that cell without being forced to walk to a different pre-assigned stand tile

#### Scenario: Complete build task
- **WHEN** construction work on the assigned site reaches completion while the colonist is adjacent
- **THEN** the finished building is created, the construction order is removed, and the colonist's task is cleared to Idle

#### Scenario: Path to build stand
- **WHEN** a colonist is assigned a Build task at a distant construction site
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the site

#### Scenario: No path to construction site
- **WHEN** no walkable path exists to an adjacent stand for the construction site
- **THEN** the Build task is cancelled, the site reservation is released, and the colonist returns to idle

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles. Other colonists SHALL NOT be treated as impassable during path computation. Pathfinding SHALL explore eight directions (four cardinal and four diagonal). Orthogonal steps SHALL cost 1; diagonal steps SHALL cost √2. A diagonal step SHALL be allowed only when the destination cell is walkable and both orthogonally adjacent cells that share an edge with the diagonal step are walkable (no corner-cutting through blocked tiles).

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles but not avoiding cells occupied by other colonists

#### Scenario: Diagonal shortcut in open area
- **WHEN** a colonist needs a path across open walkable tiles where a diagonal route is shorter than an orthogonal-only route
- **THEN** the computed path includes diagonal waypoints

#### Scenario: Corner-cutting blocked
- **WHEN** a diagonal step would cut through a corner bounded by two impassable orthogonally adjacent cells
- **THEN** that diagonal neighbor is not used and the path routes around the corner

#### Scenario: No path available
- **WHEN** no walkable path exists to the task stand tile
- **THEN** the task is cancelled and the colonist returns to idle

### Requirement: Task execution
When a colonist reaches its task destination, the simulation SHALL execute the interaction primitives defined on the target building in YAML. Arrival SHALL be determined when the path is complete and the colonist's floored grid cell matches the task target cell.

#### Scenario: Complete eat task
- **WHEN** a colonist with an Eat task arrives at its stand tile orthogonally adjacent to a `berry_bush` with berries remaining
- **THEN** the colonist's food need is restored per the interaction definition, one berry is consumed, and the task is cleared

#### Scenario: Complete sleep task
- **WHEN** a colonist with a Sleep task arrives at a `bed` tile reserved for that colonist
- **THEN** the colonist rests for the duration specified in the bed interaction definition, then sleep need is restored, the bed reservation is released, and the task is cleared

#### Scenario: Resting on bed tile
- **WHEN** a colonist is in the resting period after arriving on a reserved bed tile
- **THEN** the colonist remains on the bed tile with task kind Sleep until resting completes

#### Scenario: Vacate bed tile after sleep
- **WHEN** a colonist finishes resting on a Bed tile and a free walkable cell exists within the vacate search radius
- **THEN** the colonist walks (via pathfinding that avoids occupied cells) to the nearest such cell, searching by expanding Manhattan rings from the bed (ring 1 first, then wider rings up to `VACATE_SEARCH_RADIUS`)

#### Scenario: No vacate cell after sleep
- **WHEN** a colonist finishes resting on a Bed tile but no free walkable cell exists within the vacate search radius
- **THEN** Sleep need is still restored, the bed reservation is released, the task is cleared, and the colonist remains on the bed tile

#### Scenario: Wander after sleep vacate
- **WHEN** a colonist finishes resting, walks off the bed via the vacate path, and has no assignable Eat or Sleep task
- **THEN** the simulation assigns idle wander movement on the next assignment pass after the vacate path completes

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

### Requirement: Blocked waypoint repath
When a colonist has an active path and its next waypoint cell is occupied by another colonist's settled position, the simulation SHALL recalculate movement on the assignment pass before movement runs that tick.

#### Scenario: Eat task repaths around intermediate blocker
- **WHEN** a colonist with an Eat task is blocked on its next waypoint by another colonist and an alternate route to the eat stand exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same stand target and the Eat task is unchanged

#### Scenario: Build task repaths around intermediate blocker
- **WHEN** a colonist with a Build task is blocked on its next waypoint by another colonist and an alternate route to the build stand exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same stand target and the Build task is unchanged

#### Scenario: Deconstruct task repaths around intermediate blocker
- **WHEN** a colonist with a Deconstruct task is blocked on its next waypoint by another colonist and an alternate route to the deconstruct stand exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same stand target and the Deconstruct task is unchanged

#### Scenario: Sleep task repaths around intermediate blocker
- **WHEN** a colonist with a Sleep task is blocked on its next waypoint by another colonist and an alternate route to the bed tile exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same bed target and the Sleep task is unchanged

#### Scenario: Idle wander path cleared when blocked
- **WHEN** an idle colonist following a wander path is blocked on its next waypoint
- **THEN** the wander path is cleared so a new destination can be assigned on the same assignment pass

#### Scenario: Unreachable target clears task
- **WHEN** a colonist with an Eat, Build, Deconstruct, or Sleep task is blocked on its next waypoint and no route exists to the task target avoiding occupied cells
- **THEN** the task is cleared, reservations are released, and the colonist returns to idle assignment

#### Scenario: Occupied eat stand clears for reassignment
- **WHEN** a colonist with an Eat task is blocked on its next waypoint because that waypoint is the task target cell and it is occupied by another colonist
- **THEN** the Eat task is cleared, reservations are released, and the colonist is eligible for reassignment to another bush or stand on the same assignment pass

#### Scenario: Occupied bed clears for reassignment
- **WHEN** a colonist with a Sleep task is blocked on its next waypoint because that waypoint is the bed target cell and it is occupied by another colonist
- **THEN** the Sleep task is cleared, reservations are released, and the colonist is eligible for reassignment to another bed on the same assignment pass

#### Scenario: Build waits when goal stand occupied
- **WHEN** a colonist with a Build task is blocked on its next waypoint because that waypoint is the assigned stand target and it is occupied by another colonist
- **THEN** the Build task is unchanged and the colonist waits until the stand is free

#### Scenario: Deconstruct waits when goal stand occupied
- **WHEN** a colonist with a Deconstruct task is blocked on its next waypoint because that waypoint is the assigned stand target and it is occupied by another colonist
- **THEN** the Deconstruct task is unchanged and the colonist waits until the stand is free

### Requirement: Deconstruct target availability
A `DeconstructionSite` SHALL be assignable to a colonist only when its target tile passes a free check. The order SHALL exist immediately when placed; colonists skip unassignable sites until they become free. While waiting, the building or site remains fully functional.

#### Scenario: Wall always assignable
- **WHEN** a deconstruction site targets a `wall`
- **THEN** the site is assignable immediately if unreserved

#### Scenario: Bed not assignable while occupied
- **WHEN** a deconstruction site targets a `bed` and a colonist is on the bed tile or has a Sleep reservation on that bed
- **THEN** no colonist is assigned to that deconstruction site until the bed is free

#### Scenario: Bed assignable after vacated
- **WHEN** a deconstruction site targets a `bed` and the bed becomes unoccupied with no Sleep reservation
- **THEN** an idle colonist may be assigned a Deconstruct task for that site

#### Scenario: Berry bush not assignable while eating
- **WHEN** a deconstruction site targets a `berry_bush` and a colonist has an Eat task targeting that bush
- **THEN** no colonist is assigned to that deconstruction site until the eat task ends

#### Scenario: Construction site not assignable while building
- **WHEN** a deconstruction site targets a tile where a colonist has Build reserved or is actively building
- **THEN** no colonist is assigned to that deconstruction site until the builder reservation clears

### Requirement: Deconstruct task execution
When a colonist with a Deconstruct task is orthogonally adjacent to the deconstruction site on a valid stand tile, the simulation SHALL apply deconstruction work each tick until the site completes or the task is cleared.

#### Scenario: Work at adjacent stand
- **WHEN** a colonist with a Deconstruct task is orthogonally adjacent to the deconstruction site with no remaining path waypoints
- **THEN** deconstruction work progress increases each simulation tick

#### Scenario: Deconstructor locked in place when adjacent
- **WHEN** a colonist with a Deconstruct task reaches any orthogonally adjacent cell to the deconstruction site
- **THEN** the colonist stops moving, its path is cleared, and it deconstructs from that cell

#### Scenario: Complete deconstruct task
- **WHEN** deconstruction work on the assigned site reaches completion while the colonist is adjacent
- **THEN** the target building is removed, the deconstruction site is despawned, and the colonist's task is cleared to Idle

#### Scenario: Path to deconstruct stand
- **WHEN** a colonist is assigned a Deconstruct task at a distant deconstruction site
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the site

#### Scenario: No path to deconstruction site
- **WHEN** no walkable path exists to an adjacent stand for the deconstruction site
- **THEN** the Deconstruct task is cancelled, the site reservation is released, and the colonist returns to idle

#### Scenario: Target gone cancels deconstruct
- **WHEN** a colonist has a Deconstruct task but the deconstruction site or its target building no longer exists
- **THEN** the task is cleared and the site reservation is released

### Requirement: Berry bush pass-through without settling
Colonists SHALL be able to traverse tiles occupied by buildings with `blocks_settle: true` during movement, but SHALL NOT treat such tiles as valid settled cells or movement waypoint termini.

#### Scenario: Bush waypoint skipped during movement
- **WHEN** a colonist's next path waypoint is a `berry_bush` tile
- **THEN** the colonist does not snap to that tile and advances to the following waypoint while continuing movement

#### Scenario: Colonist ejected from bush cell
- **WHEN** a colonist's settled grid cell is a `berry_bush` tile after movement
- **THEN** the simulation moves the colonist to a nearby settleable cell if one exists

#### Scenario: Eat stand not on bush
- **WHEN** assigning an Eat or Build stand tile
- **THEN** the simulation does not select a tile with `blocks_settle: true` as the stand
