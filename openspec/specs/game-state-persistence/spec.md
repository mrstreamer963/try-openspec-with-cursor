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
The system SHALL allow the user to persist the current colony snapshot. On web, persistence SHALL trigger a browser download. On desktop, persistence SHALL write JSON to the app data `saves/` directory for the selected slot or autosave. Export SHALL allow writing a JSON file to a user-chosen path via native save dialog on desktop.

#### Scenario: Save while paused
- **WHEN** the user saves while the simulation is paused
- **THEN** the persisted file reflects `state.paused: true` and all entity positions at the moment of save

#### Scenario: Desktop slot save
- **WHEN** the user saves to slot 1 on desktop
- **THEN** `saves/slot-1.json` contains `version: 1`, `saved_at`, optional `content_mods`, and valid `state`

#### Scenario: Web download save
- **WHEN** the user saves on web
- **THEN** the browser downloads a file named with the `colony-save-` prefix and a date/time component with `.json` extension

#### Scenario: Export save on desktop
- **WHEN** the user selects File → Export Save on desktop
- **THEN** a native save dialog opens and the chosen path receives pretty-printed JSON

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
The view layer SHALL compare save `content_mods` to the currently enabled mod list when loading a save. When they differ, the user SHALL be offered three choices: load anyway with current mods, switch to the save's mod list and reload content before loading, or cancel.

#### Scenario: Matching mod lists load silently
- **WHEN** save `content_mods` equals the current enabled mod list
- **THEN** load proceeds without a mod mismatch prompt

#### Scenario: Mismatched mod lists three-way prompt
- **WHEN** save `content_mods` differs from the current enabled mod list
- **THEN** the view layer shows a dialog with saved mods, current mods, and options to load anyway, switch mods and load, or cancel

#### Scenario: Switch mods and load
- **WHEN** the user chooses switch mods and load
- **THEN** `settings.enabled_mods` is updated to match the save, content is reloaded, a new game session starts, and `load_state` applies the save

### Requirement: Save slots and autosave
The system SHALL support save identifiers `autosave`, `slot-1`, `slot-2`, and `slot-3`. Autosave SHALL be overwritten on each autosave event. Manual slots SHALL persist until overwritten by the user.

#### Scenario: Autosave file location
- **WHEN** autosave runs on desktop
- **THEN** the file is written to `saves/autosave.json`

#### Scenario: Manual slot overwrite
- **WHEN** the user saves to slot 2
- **THEN** `saves/slot-2.json` is created or replaced with the current snapshot

### Requirement: Atomic save writes
All save writes on desktop SHALL use a write-temp-then-rename pattern to avoid corrupt partial files.

#### Scenario: Atomic autosave
- **WHEN** autosave writes `autosave.json`
- **THEN** the implementation writes `autosave.json.tmp` first and renames to `autosave.json` on success

### Requirement: Autosave schedule
The application SHALL autosave to `autosave` when enabled in settings at a configurable interval (default 5 minutes) and when quitting with save from the quit guard.

#### Scenario: Interval autosave
- **WHEN** autosave is enabled and the interval elapses during an active game session
- **THEN** the current snapshot is written to `autosave.json`

#### Scenario: Autosave disabled
- **WHEN** `settings.autosave.enabled` is false
- **THEN** interval autosave does not run

### Requirement: Dirty state tracking
The application SHALL track whether game state has changed since the last successful persist. Outgoing simulation events (build, deconstruct, pause, speed, load_state) SHALL set dirty except that a successful save or load SHALL clear dirty.

#### Scenario: Dirty after build
- **WHEN** the user places a building and no save has occurred since
- **THEN** dirty is true

#### Scenario: Clean after save
- **WHEN** a save completes successfully
- **THEN** dirty is false

### Requirement: Save listing metadata
The application SHALL list available saves with `saved_at` and optional `content_mods` for display in the load UI without fully validating game state.

#### Scenario: List slots for load menu
- **WHEN** the user opens Load Game
- **THEN** existing slot and autosave files are listed with timestamp and mod list when present

### Requirement: Web persistence fallback
On web, saves and settings SHALL persist via `localStorage` or IndexedDB using the same `SaveId` keys, emulating slot and autosave behavior without native filesystem access.

#### Scenario: Web autosave in storage
- **WHEN** autosave runs on web
- **THEN** the save JSON is stored under a stable storage key and retrievable on next visit
