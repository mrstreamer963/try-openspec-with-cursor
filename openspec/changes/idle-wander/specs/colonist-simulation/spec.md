## MODIFIED Requirements

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
- **THEN** the colonist remains idle and the simulation assigns idle wander movement (see Idle wander requirement)

## ADDED Requirements

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
