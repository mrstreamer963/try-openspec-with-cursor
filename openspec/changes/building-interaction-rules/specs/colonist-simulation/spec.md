## MODIFIED Requirements

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a BerryBush with berries remaining exists with at least one adjacent walkable stand tile
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush, with path destination on an adjacent stand tile

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist's Sleep need drops below the threshold and an unoccupied Bed exists
- **THEN** the colonist is assigned a Sleep task targeting the nearest such Bed and the bed is reserved for that colonist

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold
- **THEN** the colonist remains idle with no assigned task

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles.

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles

#### Scenario: No path available
- **WHEN** no walkable path exists to the task stand tile
- **THEN** the task is cancelled and the colonist returns to idle

### Requirement: Task execution
When a colonist reaches its task destination, the simulation SHALL execute the task interaction and restore the relevant need.

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

## ADDED Requirements

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
