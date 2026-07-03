## MODIFIED Requirements

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
- **THEN** the colonist remains idle with no assigned task
