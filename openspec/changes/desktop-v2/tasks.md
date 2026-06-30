## 1. Tauri spike (gate)

- [ ] 1.1 Run `tauri init` at repo root; configure `frontendDist` to `packages/client/dist` and dev URL to Vite port 5173
- [ ] 1.2 Add root scripts `dev:desktop` and `build:desktop`; add `@tauri-apps/api` and CLI deps
- [ ] 1.3 Adjust `tauri.conf.json` CSP for WASM worker; verify YAML fetch, worker start, and WASM init in `tauri dev` on macOS

## 2. Platform adapter

- [ ] 2.1 Create `packages/client/src/platform/types.ts` with `SaveStorage`, `DesktopShell`, `NativeUi`, and `PlatformAdapter`
- [ ] 2.2 Implement `webAdapter.ts` (download, file input, confirm, localStorage saves/settings)
- [ ] 2.3 Implement `tauriAdapter.ts` using `plugin-fs`, `plugin-dialog`, `plugin-shell`
- [ ] 2.4 Add `getPlatform()` factory and `isDesktop()` helper; wire Tauri capabilities scoped to app data

## 3. Settings persistence

- [ ] 3.1 Define `AppSettings` type (`enabled_mods`, `autosave`, `window`, `last_slot`)
- [ ] 3.2 Implement `loadSettings()` / `saveSettings()` via platform adapter
- [ ] 3.3 Default `enabled_mods: ["base"]`; ensure first launch creates app data dirs and `mods/README.txt`

## 4. Content loading refactor

- [ ] 4.1 Add `ContentSource` interface and `BundledContentSource` using `import.meta.env.BASE_URL`
- [ ] 4.2 Add `UserModContentSource` (Tauri fs reads under `app_data/mods/`)
- [ ] 4.3 Refactor `loadContent.ts` to accept `enabledModIds` and sources; remove hard dependency on `mods.yaml` for runtime order
- [ ] 4.4 Implement mod catalog discovery (bundled from `content/mods.yaml` + user folder scan)
- [ ] 4.5 Update `loadContent` tests for new API; add tests for catalog merge (user overrides bundled id)

## 5. Save storage

- [ ] 5.1 Implement `SaveStorage` with `SaveId` (`autosave`, `slot-1`..`slot-3`) and atomic write-temp-rename
- [ ] 5.2 Implement `listSaves()` returning metadata (`saved_at`, `content_mods`)
- [ ] 5.3 Wire `buildSaveFile` / `validateSaveFile` through storage (format unchanged)
- [ ] 5.4 Add export via native save dialog on desktop; keep web download for export

## 6. App state machine and session lifecycle

- [ ] 6.1 Extract `GameSession` component from `App.vue` (GameManager, PixiRenderer, HUD, keyboard handlers)
- [ ] 6.2 Add top-level state: `menu | loading | playing`; main menu does not start worker
- [ ] 6.3 Implement `startNewGame()`, `continueFromAutosave()`, `loadFromSave()`, `quitToMenu()`
- [ ] 6.4 Implement `restartGame(modIds)` for switch-mods-and-load flow

## 7. Main menu and mod picker UI

- [ ] 7.1 Add `MainMenu.vue` (Continue, New Game, Load Game, Mods, Quit)
- [ ] 7.2 Add `ModPicker.vue` with bundled/user mod list, toggles, Apply, Open Mods Folder
- [ ] 7.3 Add `LoadGameScreen.vue` with slot list and import file option
- [ ] 7.4 Continue button enabled only when valid autosave exists

## 8. Save/load UX and mod mismatch

- [ ] 8.1 Update `Hud.vue` Save/Load to use slot picker and quick-save slot from settings
- [ ] 8.2 Implement three-way mod mismatch dialog (load anyway / switch mods & load / cancel)
- [ ] 8.3 Implement dirty tracking; clear on successful save/load
- [ ] 8.4 Implement autosave interval timer and quit-guard save prompt on desktop

## 9. Native menu bar

- [ ] 9.1 Configure Tauri menu: File (Save, Load, Export, Quit), Mods (Manage, Open Folder)
- [ ] 9.2 Wire menu items to same handlers as in-app UI
- [ ] 9.3 Handle `CloseRequested` window event with quit guard

## 10. Build and verification

- [ ] 10.1 Ensure `build:desktop` runs wasm → client → tauri in order
- [ ] 10.2 Manual test matrix on macOS: New Game, Save slots, Autosave, Continue, Load, mod toggle, user mod folder, switch mods & load, quit guard
- [ ] 10.3 Verify web build still works with main menu and web adapter fallback
