## MODIFIED Requirements

### Requirement: Automatic task assignment
When a colonist's need drops below the critical threshold, the simulation SHALL automatically assign a task to satisfy that need.

#### Scenario: Auto-assign eat task
- **WHEN** a colonist's Food need drops below the threshold and a BerryBush with berries remaining exists
- **THEN** the colonist is assigned an Eat task targeting the nearest such BerryBush

## ADDED Requirements

### Requirement: Eat fails on depleted bush
When a colonist completes an Eat task at a tile with no berries, the simulation SHALL not restore Food and SHALL clear the task.

#### Scenario: Bush depleted before arrival
- **WHEN** a colonist arrives at a BerryBush tile that has no berries (or no bush)
- **THEN** the colonist's Food need is unchanged and the task is cleared
