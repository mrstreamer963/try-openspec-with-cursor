# Game State Persistence

## Purpose

Versioned JSON save files, client-side validation, and save/load user flows for preserving colony progress across sessions.

## Requirements

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

### Requirement: Deconstruction state round-trip
Save and load SHALL preserve pending deconstruction sites so colonists resume work after a session reload.

#### Scenario: Save includes deconstruction sites
- **WHEN** the user saves while deconstruction sites are pending
- **THEN** the save file's `state.deconstruction_sites` array contains each site's position, building id, and progress

#### Scenario: Load restores deconstruction sites
- **WHEN** the user loads a save file with pending deconstruction sites
- **THEN** the simulation restores `DeconstructionSite` entities and colonists can be assigned to complete them

### Requirement: Content mods in save file
Save files MAY include an optional `content_mods` string array recording the mod ids active when the game was saved. When present, entries SHALL be ordered mod ids from the manifest (e.g. `["base", "hardmode"]`).

#### Scenario: Save records active mods
- **WHEN** the user saves with mods `base` and `hardmode` active
- **THEN** the downloaded JSON includes `content_mods: ["base", "hardmode"]`

#### Scenario: Legacy save without content_mods
- **WHEN** the user loads a save file with no `content_mods` field
- **THEN** load proceeds treating the save as created with `["base"]` only

### Requirement: Mod mismatch warning on load
The view layer SHALL compare save `content_mods` to the currently loaded mod list when the user loads a save file. When they differ, the user SHALL be prompted to confirm before applying the save.

#### Scenario: Matching mod lists load silently
- **WHEN** save `content_mods` equals the current loaded mod list
- **THEN** load proceeds without a mod mismatch prompt

#### Scenario: Mismatched mod lists prompt user
- **WHEN** save `content_mods` differs from the current loaded mod list
- **THEN** the view layer shows a confirmation dialog describing both lists before calling `load_state`
