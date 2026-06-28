# Colonist Simulation

## Purpose

Simulates colonist entities: needs, automatic task assignment, pathfinding, and task execution.

## Requirements

### Requirement: Three starting colonists
The simulation SHALL spawn exactly 3 colonists at game start on walkable tiles.

#### Scenario: Initial colonist count
- **WHEN** the game initializes
- **THEN** exactly 3 colonist entities exist with unique IDs and valid positions

### Requirement: Colonist needs
Each colonist SHALL have two needs: Food and Sleep, each represented as a value from 0 (critical) to 100 (satisfied).

#### Scenario: Needs decay over time
- **WHEN** the simulation ticks while a colonist is idle
- **THEN** both Food and Sleep values decrease at a configurable rate

#### Scenario: Critical need threshold
- **WHEN** a colonist's Food or Sleep value drops below a defined threshold (e.g., 30)
- **THEN** the colonist is flagged as needing that resource

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a BerryBush exists
- **THEN** the colonist is assigned an Eat task targeting the nearest BerryBush

#### Scenario: Auto-assign sleep task
- **WHEN** a colonist's Sleep need drops below the threshold and a Bed exists
- **THEN** the colonist is assigned a Sleep task targeting the nearest Bed

#### Scenario: No task when needs are satisfied
- **WHEN** all of a colonist's needs are above the threshold
- **THEN** the colonist remains idle with no assigned task

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles.

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path avoiding impassable tiles

#### Scenario: No path available
- **WHEN** no walkable path exists to the task destination
- **THEN** the task is cancelled and the colonist returns to idle

### Requirement: Task execution
When a colonist reaches its task destination, the simulation SHALL execute the task interaction and restore the relevant need.

#### Scenario: Complete eat task
- **WHEN** a colonist with an Eat task arrives at a BerryBush tile
- **THEN** the colonist's Food need is restored and the task is cleared

#### Scenario: Complete sleep task
- **WHEN** a colonist with a Sleep task arrives at a Bed tile
- **THEN** the colonist's Sleep need is restored and the task is cleared
