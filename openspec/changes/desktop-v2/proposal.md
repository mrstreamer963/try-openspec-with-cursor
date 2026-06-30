## Why

The game runs as a browser SPA with download-based saves, a hand-edited mod manifest, and no session continuity. Shipping a Tauri desktop build with native persistence, mod management, and a main menu turns the prototype into a product players can install and return to—without abandoning the existing web dev workflow.

## What Changes

- Add Tauri 2 shell at repo root wrapping the Vite client (`src-tauri/`, build pipeline, CSP validated for WASM workers)
- Introduce `PlatformAdapter` abstraction: Tauri (fs, dialog, shell) on desktop; existing download/file-input behavior on web
- Persist saves in app data: `autosave.json` + three manual slots (`slot-1`..`slot-3`); atomic writes; optional export via native dialog
- Add `settings.json` in app data: `enabled_mods`, autosave interval, window geometry, last slot
- Main menu on startup: Continue (autosave), New Game, Load Game, Mods, Quit
- Mod picker UI: toggle bundled mods; discover user mods from `app_data/mods/`; **Open Mods Folder** with README
- Refactor `loadContent()` to use pluggable `ContentSource` (bundled fetch + user-mod filesystem on desktop)
- Dirty tracking + quit guard (save before close when state changed since last persist)
- Load with save's mods: three-way dialog (load anyway / switch mods & reload / cancel)
- Application menu bar: File (Save, Load, Export, Quit), Mods (Manage, Open Folder)
- **BREAKING (desktop UX)**: game no longer auto-starts on launch; user enters via main menu. Web keeps equivalent main menu flow.

## Capabilities

### New Capabilities

- `desktop-shell`: Tauri app shell, platform adapter, app data paths, native menus, quit guard, build pipeline

### Modified Capabilities

- `content-definitions`: settings-driven enabled mod list; bundled catalog vs runtime manifest; user mod directory discovery and load via filesystem source; user mod overrides bundled by id
- `game-state-persistence`: app-data slot saves and autosave; dirty tracking; atomic persist; export; web fallback (localStorage/IndexedDB); load-with-save's-mods flow
- `view-layer`: main menu screen; mod picker; slot-based save/load UI; native confirm dialogs; menu bar integration

## Impact

- `src-tauri/` — new Tauri project (Cargo.toml, tauri.conf.json, capabilities)
- Root `package.json` — `dev:desktop`, `build:desktop` scripts; `@tauri-apps/*` deps
- `packages/client/src/platform/` — adapter interfaces and web/tauri implementations
- `packages/client/src/content/loadContent.ts` — ContentSource abstraction
- `packages/client/src/App.vue` — state machine (menu | playing), session lifecycle, dirty tracking
- `packages/client/src/components/` — MainMenu, ModPicker, SaveLoadDialog (new)
- `packages/client/src/game/saveFile.ts` — unchanged format; transport moves to adapter
- `content/mods.yaml` — becomes shipped-mod catalog reference, not sole runtime config
- No WASM API changes
