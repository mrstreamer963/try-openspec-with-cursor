## ADDED Requirements

### Requirement: Blocked waypoint repath
When a colonist has an active path and its next waypoint cell is occupied by another colonist's settled position, the simulation SHALL recalculate movement on the assignment pass before movement runs that tick.

#### Scenario: Eat task repaths around intermediate blocker
- **WHEN** a colonist with an Eat task is blocked on its next waypoint by another colonist and an alternate route to the eat stand exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same stand target and the Eat task is unchanged

#### Scenario: Build task repaths around intermediate blocker
- **WHEN** a colonist with a Build task is blocked on its next waypoint by another colonist and an alternate route to the build stand exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same stand target and the Build task is unchanged

#### Scenario: Sleep task repaths around intermediate blocker
- **WHEN** a colonist with a Sleep task is blocked on its next waypoint by another colonist and an alternate route to the bed tile exists
- **THEN** the colonist's path waypoints are replaced with a new route to the same bed target and the Sleep task is unchanged

#### Scenario: Idle wander path cleared when blocked
- **WHEN** an idle colonist following a wander path is blocked on its next waypoint
- **THEN** the wander path is cleared so a new destination can be assigned on the same assignment pass

#### Scenario: Unreachable target clears task
- **WHEN** a colonist with an Eat, Build, or Sleep task is blocked on its next waypoint and no route exists to the task target avoiding occupied cells
- **THEN** the task is cleared, reservations are released, and the colonist returns to idle assignment

#### Scenario: Occupied goal clears task for reassignment
- **WHEN** a colonist with an Eat, Build, or Sleep task is blocked on its next waypoint because that waypoint is the task target cell and it is occupied by another colonist
- **THEN** the task is cleared, reservations are released, and the colonist is eligible for reassignment on the same assignment pass

#### Scenario: Eat reassigned to alternate stand or bush
- **WHEN** a hungry colonist's Eat task is cleared because its stand target is occupied and another satisfiable BerryBush or free stand exists
- **THEN** the colonist is assigned a new Eat task to that alternate target on the same assignment pass

#### Scenario: Build reassigned after occupied stand
- **WHEN** a colonist's Build task is cleared because its stand target is occupied and another construction site with a free stand exists
- **THEN** the colonist may be assigned a new Build task on the same assignment pass
