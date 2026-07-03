## ADDED Requirements

### Requirement: HUD save and load controls
The view layer SHALL display Save and Load buttons in the HUD alongside existing pause and speed controls.

#### Scenario: Save button triggers download
- **WHEN** the user clicks Save in the HUD
- **THEN** the application downloads a version-1 JSON save file for the current colony

#### Scenario: Load button opens file picker
- **WHEN** the user clicks Load in the HUD
- **THEN** a native file picker opens filtered to `.json` files

#### Scenario: Load error feedback
- **WHEN** the selected save file fails validation or the worker reports a load error
- **THEN** the view layer displays an error message to the user without crashing the game view

#### Scenario: Renderer updates after load
- **WHEN** a load succeeds and a new snapshot arrives
- **THEN** the PixiJS scene updates to show the restored world on the next animation frame
