## MODIFIED Requirements

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

## ADDED Requirements

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
