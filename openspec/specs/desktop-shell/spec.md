# Desktop Shell

## Purpose

Tauri 2 desktop application shell, platform adapter abstraction, app data layout, native menus, and quit-guard behavior for the idle colony sim client.

## Requirements

### Requirement: Tauri application shell
The repository SHALL include a Tauri 2 application at `src-tauri/` that loads the Vite client build as its webview frontend. Root npm scripts SHALL provide `dev:desktop` (Tauri dev with Vite HMR) and `build:desktop` (WASM build, client build, then `tauri build`).

#### Scenario: Desktop dev starts
- **WHEN** the developer runs `npm run dev:desktop`
- **THEN** the Tauri window opens, loads the client, and content YAML fetch plus WASM worker initialization succeed

#### Scenario: Desktop release build
- **WHEN** the developer runs `npm run build:desktop`
- **THEN** a platform-native application bundle is produced containing the client dist, bundled content, and WASM assets

### Requirement: Platform adapter abstraction
The client SHALL route filesystem, dialog, and persistence operations through a `PlatformAdapter` interface with separate web and Tauri implementations. Application code outside the adapter SHALL NOT import Tauri APIs directly.

#### Scenario: Web build uses web adapter
- **WHEN** the client runs in a browser without Tauri
- **THEN** save, load, confirm, and settings operations use browser APIs (download, file input, `window.confirm`, `localStorage`)

#### Scenario: Tauri build uses desktop adapter
- **WHEN** the client runs inside the Tauri webview
- **THEN** save, load, confirm, and settings operations use Tauri plugins (`fs`, `dialog`, `shell`)

### Requirement: App data directory layout
On desktop, the application SHALL store user data under the Tauri app data directory in a subdirectory named `idle-colony-sim` with at minimum: `settings.json`, `saves/`, and `mods/`.

#### Scenario: First launch creates directories
- **WHEN** the desktop app launches and app data paths do not exist
- **THEN** the application creates `saves/` and `mods/` and writes default `settings.json`

#### Scenario: Mods folder README
- **WHEN** the user opens the mods folder for the first time
- **THEN** a `README.txt` is present describing mod folder structure (`mod.yaml`, optional category YAML files)

### Requirement: Native application menu
The desktop application SHALL provide a menu bar with at minimum: **File** (Save, Load, Export Save, Quit) and **Mods** (Manage Mods, Open Mods Folder). Menu actions SHALL invoke the same logic as in-app UI controls.

#### Scenario: File Save from menu
- **WHEN** the user selects File → Save while a game session is active
- **THEN** the current colony is written to the configured quick-save slot

#### Scenario: Open Mods Folder from menu
- **WHEN** the user selects Mods → Open Mods Folder
- **THEN** the OS file manager opens the app data `mods/` directory

### Requirement: Quit guard on unsaved changes
The desktop application SHALL intercept window close while a game session is active and `dirty` is true. The user SHALL be prompted to save (autosave), quit without saving, or cancel.

#### Scenario: Clean quit
- **WHEN** the user closes the window and no unsaved changes exist since the last persist
- **THEN** the application exits without prompting

#### Scenario: Dirty quit prompt
- **WHEN** the user closes the window after changing game state without a subsequent save
- **THEN** a native confirm dialog offers save, discard, and cancel options

#### Scenario: Save and quit
- **WHEN** the user chooses save from the quit prompt
- **THEN** `autosave.json` is written and the application exits

### Requirement: Cross-platform app data paths
The desktop shell SHALL use Tauri path APIs (`appDataDir`, `join`) for all user data paths. No hardcoded macOS-specific paths SHALL appear in client TypeScript.

#### Scenario: Path resolution on macOS
- **WHEN** the app runs on macOS
- **THEN** saves are stored under `~/Library/Application Support/idle-colony-sim/saves/`

#### Scenario: Path resolution uses Tauri API
- **WHEN** the desktop adapter resolves the saves directory
- **THEN** it calls Tauri `path.appDataDir()` and joins `saves` rather than constructing a platform string manually

### Requirement: Filesystem capability scope
Tauri `fs` permissions SHALL be scoped to the application data directory only. The frontend SHALL NOT request broad filesystem access.

#### Scenario: Write save within scope
- **WHEN** the application writes `saves/autosave.json`
- **THEN** the operation succeeds under the configured fs scope

#### Scenario: Arbitrary path denied
- **WHEN** client code attempts to write outside the app data scope
- **THEN** the Tauri runtime denies the operation
