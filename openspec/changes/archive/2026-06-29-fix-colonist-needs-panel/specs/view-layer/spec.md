## MODIFIED Requirements

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing the colonist's display name, numeric id, needs, current task, and position. Position SHALL display the colonist's grid cell (floored coordinates). For each need (Food and Sleep), the panel SHALL show the numeric value, a progress bar, and a critical status label when that need is below the simulation threshold.

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing the colonist's name and id (e.g. `Mira (#2)`), Food value with bar, Sleep value with bar, current task name, and grid coordinates

#### Scenario: Critical need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: true`
- **THEN** the Food row displays a visible "Hungry" status label in addition to the numeric value and bar

#### Scenario: Sleep need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `wants_sleep: true`
- **THEN** the Sleep row displays a visible "Wants sleep" status label in addition to the numeric value and bar

#### Scenario: Satisfied needs show no critical label
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: false` and `wants_sleep: false`
- **THEN** neither need row shows a critical status label

#### Scenario: Close info panel
- **WHEN** the user clicks elsewhere on the canvas
- **THEN** the colonist info panel is dismissed
