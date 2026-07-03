## MODIFIED Requirements

### Requirement: Versioned save file format
The system SHALL persist game state as a JSON file with structure `{ version, saved_at, state }` where `version` is `1`, `saved_at` is an ISO-8601 timestamp string, and `state` is a `StateSnapshot` matching the game core snapshot schema.

#### Scenario: Valid save file structure
- **WHEN** the user saves the game
- **THEN** the downloaded file contains `version: 1`, a `saved_at` timestamp, and `state` with tiles, buildings, construction_sites, deconstruction_sites, colonists, paused, and speed fields

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

## ADDED Requirements

### Requirement: Deconstruction state round-trip
Save and load SHALL preserve pending deconstruction sites so colonists resume work after a session reload.

#### Scenario: Save includes deconstruction sites
- **WHEN** the user saves while deconstruction sites are pending
- **THEN** the save file's `state.deconstruction_sites` array contains each site's position, building id, and progress

#### Scenario: Load restores deconstruction sites
- **WHEN** the user loads a save file with pending deconstruction sites
- **THEN** the simulation restores `DeconstructionSite` entities and colonists can be assigned to complete them
