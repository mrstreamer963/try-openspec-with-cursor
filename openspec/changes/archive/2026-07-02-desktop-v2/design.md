## Context

The client is a Vue 3 + Vite SPA with PixiJS rendering and a WASM game core in a Web Worker. Content loads via HTTP `fetch` from Vite `publicDir` (`content/`). Saves use browser download and a hidden file input. Mods are listed in `content/mods.yaml` and merged at startup. Multi-mod loading and `content_mods` in saves are already implemented.

Desktop v2 wraps the same frontend in Tauri 2 while adding native persistence, mod management, and session flow—without forking the web build.

## Goals / Non-Goals

**Goals:**

- Tauri 2 shell with `tauri dev` / `tauri build` integrated into root npm scripts
- `PlatformAdapter` with web and Tauri implementations (single client codebase)
- App data layout: `settings.json`, `saves/` (autosave + 3 slots), `mods/` (user-installed)
- Main menu → game session state machine; Continue from autosave
- Mod picker: toggle bundled mods; discover user mods; Open Mods Folder
- `ContentSource` abstraction: bundled `fetch` + optional user-mod `fs` read
- Dirty tracking + quit guard; autosave on interval and close
- Load with save's mods (switch enabled mods, reload content, restart session)
- Native menu bar (File, Mods) on desktop
- macOS-first development and manual QA; cross-platform Tauri build config from day one

**Non-Goals:**

- Auto-updater, code signing, notarization, Steam
- Win/Linux CI matrix (manual builds acceptable for v2)
- In-game mod hot-reload during simulation
- Zip mod install / Steam Workshop
- Deep-merge of individual YAML fields within one definition
- Changing save file `version` or WASM API

## Decisions

### 1. Tauri at repo root (`src-tauri/`)

**Choice:** Standard Tauri layout with `frontendDist` pointing to `packages/client/dist`.

**Alternatives:** `packages/desktop/` wrapper — rejected; adds path indirection without benefit at current scale.

**Build order:** `build:wasm` → `build -w packages/client` → `tauri build`.

### 2. PlatformAdapter (three interfaces)

```text
SaveStorage   — list/read/write/delete saves by SaveId
DesktopShell  — app paths, readText, listDir, openModsFolder
NativeUi      — confirm, pickOpenFile, pickSaveFile
```

Detection: `window.__TAURI_INTERNALS__` or `@tauri-apps/api/core` `isTauri()`. Web uses `downloadSaveFile`, `<input type="file">`, `window.confirm`, and `localStorage`/`IndexedDB` for saves/settings.

### 3. App data paths (cross-platform via Tauri)

Tauri `path.appDataDir()` resolves per OS:

| OS      | Example                                      |
|---------|----------------------------------------------|
| macOS   | `~/Library/Application Support/idle-colony-sim/` |
| Windows | `%APPDATA%/idle-colony-sim/`                 |
| Linux   | `~/.local/share/idle-colony-sim/`              |

**macOS-first** means primary dev/QA on darwin; no platform-specific code in TypeScript beyond adapter—Tauri APIs are cross-platform. Differences to expect later (not v2 blockers):

- Menu bar integration differs (macOS app menu vs Windows in-window menu)
- Window close event handling (same API, different UX expectations)
- WebView2 must be installed on Windows (Tauri prerequisite)
- Linux may need `webkit2gtk` dev packages for local builds

### 4. SaveId and atomic writes

```text
SaveId = 'autosave' | 'slot-1' | 'slot-2' | 'slot-3'
```

Write pattern: `*.json.tmp` → rename to target. Prevents corrupt autosave on crash mid-write.

### 5. settings.json drives runtime mods

`enabled_mods: string[]` replaces `content/mods.yaml` as runtime config. `content/mods.yaml` remains a **catalog** of shipped optional mods (ids discoverable in mod picker). `base` is always first and not toggleable.

Default: `["base"]` (hardmode off, matching pre-desktop default intent).

### 6. User mod discovery

Scan `app_data/mods/*/mod.yaml`. User mod with same `id` as bundled mod **overrides** bundled metadata and YAML (user source wins on merge). Validation reuses existing `loadModPartial` rules.

Web: user mod folder not available; mod picker shows bundled mods only.

### 7. ContentSource abstraction

```text
interface ContentSource {
  readText(path: string): Promise<string>;
  exists?(path: string): Promise<boolean>;
}
```

- `BundledContentSource` — `fetch` with `import.meta.env.BASE_URL` prefix (fixes Tauri asset protocol)
- `UserModContentSource` — Tauri `fs.readTextFile` under `app_data/mods/<id>/`

`loadContent(enabledModIds, sources)` unchanged merge order: base → enabled mods in settings order.

### 8. App state machine

```text
menu ──(New/Continue/Load)──► loading ──► playing
playing ──(Quit to menu)──► menu
```

`restartGame(modIds)` tears down `GameManager` + `PixiRenderer`, clears content cache, reloads content, re-inits. Used for "Switch mods & load".

### 9. Dirty tracking

Set `dirty = true` on any outgoing `sendEvent` except read-only queries. Set `dirty = false` after successful `writeSave`. Quit guard prompts when `dirty && playing`.

### 10. Mod mismatch: three-way dialog

When save `content_mods` ≠ current `enabled_mods`:

1. **Load anyway** — current behavior (`load_state` with active content)
2. **Switch mods & load** — update settings, `restartGame(save.content_mods)`, then `load_state`
3. **Cancel**

### 11. CSP and WASM spike (task 1 gate)

Before feature work: `tauri dev` smoke test — YAML fetch, worker start, WASM init. Adjust `tauri.conf.json` CSP (`wasm-unsafe-eval`, worker-src) if needed. **Do not proceed** if spike fails.

### 12. Tauri capabilities scope

`fs` scope limited to `$APPDATA/**` (app data dir). No arbitrary filesystem access from frontend.

## Risks / Trade-offs

- **[restartGame lifecycle bugs]** → Explicit teardown in `GameSession` component; manual test matrix for Continue/Load/Switch mods
- **[fetch paths in Tauri]** → Use `BASE_URL`-relative paths in `BundledContentSource`; spike first
- **[Web parity maintenance]** → Adapter pattern; web tests unchanged; desktop tests manual initially
- **[User mod bad YAML]** → Same error surfaces as bundled fetch failures; block at loading screen
- **[Scope]** → No updater/signing; defer Win/Linux CI
- **[Main menu breaks dev flow]** → `tauri dev` still uses menu; acceptable one extra click

## Migration Plan

1. Ship Tauri alongside web; web users see main menu but behavior is familiar
2. Existing browser-downloaded saves importable via Load file / Export
3. `content/mods.yaml` can list all shipped mods for catalog; default `settings.enabled_mods: ["base"]`
4. No save version bump; `content_mods` field unchanged

## Open Questions

_None — scope locked at desktop v2._
