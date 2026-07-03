## Why

The base content pack currently exists in two places: split YAML files under `content/base/` (used by the client) and a merged `pack.json` (embedded in Rust tests via `include_str!`). Keeping them in sync is manual and error-prone. The client also loads YAML through Vite `?raw` imports, which bundles content at build time rather than reading the live files at startup. We want a single authoritative YAML source loaded dynamically when the app boots (page refresh is sufficient; no mid-game reload).

## What Changes

- Remove `content/base/pack.json` as a duplicate source of truth
- Replace client `?raw` YAML imports with **runtime `fetch`** of `content/base/*.yaml` at application startup
- Serve `content/base/` as static assets in dev and production (no build scripts to generate JSON)
- Make content loading **async**: show loading screen until YAML is fetched, parsed, and merged into a `ContentPack`
- Pass the merged content from the main thread to the worker (or have worker fetch the same URLs once) before `Game::new(contentJson)`
- Update UI components (`Toolbar`, `PixiRenderer`, `ColonistInfo`) to receive content after async load instead of calling `loadBaseContent()` at module import time
- Update Rust unit/integration tests to read the same YAML files from disk (via `serde_yaml` or a small test helper) instead of `include_str!("pack.json")`
- **Out of scope**: hot-reloading content during an active simulation; content changes apply only after a full page refresh

## Capabilities

### New Capabilities

_None — this change tightens the existing content-loading architecture rather than introducing a new capability domain._

### Modified Capabilities

- `content-definitions`: YAML files are the sole authoritative source; loading happens at application startup via HTTP fetch, not compile-time bundle or embedded JSON
- `view-layer`: UI waits for async content load before rendering toolbar, colors, and colonist labels
- `worker-bridge`: Worker receives content payload from the boot sequence after YAML fetch/merge; no embedded `pack.json` fallback

## Impact

- `content/base/` — remove `pack.json`; YAML files remain the only content definitions
- `packages/client/src/content/loadBaseContent.ts` — async `fetch`-based loader; remove `?raw` imports
- `packages/client/vite.config.ts` — serve `content/` as static assets (e.g. `public/` copy or static dir config)
- `packages/client/src/App.vue` — orchestrate async content boot before starting worker/game
- `packages/client/src/components/Toolbar.vue`, `ColonistInfo.vue` — accept injected `ContentPack` instead of sync module-level load
- `packages/client/src/game/PixiRenderer.ts` — receive content at construction time
- `packages/client/src/worker/gameWorker.ts` — accept content JSON in `start` message or fetch once at init
- `packages/game-core/src/content/mod.rs` — remove `base_content_json()` / `include_str!(pack.json)`; tests load YAML from filesystem
- `packages/game-core/Cargo.toml` — add `serde_yaml` dev-dependency for test helpers (optional: use in test-only module)
