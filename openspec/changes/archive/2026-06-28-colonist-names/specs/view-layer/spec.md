## MODIFIED Requirements

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing the colonist's display name, numeric id, needs, current task, and position.

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing the colonist's name and id (e.g. `Mira (#2)`), Food value, Sleep value, current task name, and grid coordinates

#### Scenario: Close info panel
- **WHEN** the user clicks elsewhere on the canvas
- **THEN** the colonist info panel is dismissed

## ADDED Requirements

### Requirement: Colonist name labels
The view layer SHALL render each colonist's display name as a text label positioned above the colonist sprite at all times.

#### Scenario: Name above sprite
- **WHEN** a colonist is visible on the canvas
- **THEN** its display name is drawn centered above the colonist sprite and moves with the sprite

#### Scenario: Name from snapshot
- **WHEN** a state snapshot updates a colonist's position
- **THEN** the name label follows the colonist's rendered position
