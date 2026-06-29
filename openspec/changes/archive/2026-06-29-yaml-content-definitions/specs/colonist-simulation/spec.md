## MODIFIED Requirements

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

#### Scenario: No need task when nothing satisfiable
- **WHEN** a colonist has one or more active critical statuses but no satisfiable target and valid path exist
- **THEN** the colonist remains idle with no Eat or Sleep task assigned and the simulation assigns idle wander movement if the colonist has no remaining path waypoints

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

### Requirement: Idle wander
When a colonist is idle with no Eat, Sleep, or Build assignment and no active path, the simulation SHALL assign a path to a random nearby walkable cell within a configurable wander radius. Wander destination selection SHALL exclude tiles whose building definition has `blocks_settle: true` (base pack: `berry_bush`).

#### Scenario: Wander avoids bush destination
- **WHEN** selecting a wander destination
- **THEN** the simulation does not select a tile occupied by a building with `blocks_settle: true`

### Requirement: Berry bush pass-through without settling
Colonists SHALL be able to traverse tiles occupied by buildings with `blocks_settle: true` during movement, but SHALL NOT treat such tiles as valid settled cells or movement waypoint termini.

#### Scenario: Bush waypoint skipped during movement
- **WHEN** a colonist's next path waypoint is a `berry_bush` tile
- **THEN** the colonist does not snap to that tile and advances to the following waypoint while continuing movement

#### Scenario: Colonist ejected from bush cell
- **WHEN** a colonist's settled grid cell is a `berry_bush` tile after movement
- **THEN** the simulation moves the colonist to a nearby settleable cell if one exists
