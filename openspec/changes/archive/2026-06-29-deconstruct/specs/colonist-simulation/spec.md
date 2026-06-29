## MODIFIED Requirements

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

## ADDED Requirements

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
