## MODIFIED Requirements

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a completed BerryBush with berries remaining exists
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist's Sleep need drops below the threshold and a completed Bed exists
- **THEN** the colonist is assigned a Sleep task targeting the nearest Bed

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold and no construction orders are unassigned
- **THEN** the colonist remains idle with no assigned task

## ADDED Requirements

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
