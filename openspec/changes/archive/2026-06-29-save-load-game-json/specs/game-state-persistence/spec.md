## ADDED Requirements

### Requirement: Versioned save file format
The system SHALL persist game state as a JSON file with structure `{ version, saved_at, state }` where `version` is `1`, `saved_at` is an ISO-8601 timestamp string, and `state` is a `StateSnapshot` matching the game core snapshot schema.

#### Scenario: Valid save file structure
- **WHEN** the user saves the game
- **THEN** the downloaded file contains `version: 1`, a `saved_at` timestamp, and `state` with tiles, buildings, construction_sites, colonists, paused, and speed fields

#### Scenario: Pretty-printed JSON
- **WHEN** the save file is written
- **THEN** the JSON is formatted with indentation for human readability

### Requirement: Save file validation on load
The system SHALL validate a save file before applying it: `version` must be `1`, `state` must be present, `state.tiles` must contain exactly 2500 entries (50×50 world), and all required snapshot arrays must be present.

#### Scenario: Reject unsupported version
- **WHEN** the user selects a file with `version` not equal to `1`
- **THEN** load is aborted, an error message is shown, and the current simulation is unchanged

#### Scenario: Reject malformed JSON
- **WHEN** the user selects a file that is not valid JSON
- **THEN** load is aborted with a parse error message and the current simulation is unchanged

#### Scenario: Reject incomplete state
- **WHEN** the user selects a file missing `state.tiles` or with tile count other than 2500
- **THEN** load is aborted with a validation error and the current simulation is unchanged

### Requirement: Save downloads current snapshot
The system SHALL allow the user to save the current colony by downloading a JSON file derived from the latest `StateSnapshot` held on the main thread.

#### Scenario: Save while paused
- **WHEN** the user clicks Save while the simulation is paused
- **THEN** the downloaded file reflects `state.paused: true` and all entity positions at the moment of save

#### Scenario: Save filename
- **WHEN** a save completes successfully
- **THEN** the browser downloads a file named with the `colony-save-` prefix and a date/time component with `.json` extension

### Requirement: Load replaces running simulation
The system SHALL replace the active simulation with the validated `state` from a save file and broadcast the restored snapshot to the view layer.

#### Scenario: Successful load
- **WHEN** the user selects a valid save file
- **THEN** the game core restores world state from `state`, the worker posts the new snapshot, and the renderer shows the loaded colony

#### Scenario: Load preserves pause and speed
- **WHEN** a save file has `state.paused: true` and `state.speed: 5`
- **THEN** after load the simulation remains paused at 5× speed as reflected in the HUD
