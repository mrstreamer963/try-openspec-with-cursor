## Context

The base content pack lives in `content/base/` as split YAML files (`needs.yaml`, `statuses.yaml`, `buildings.yaml`, `terrain.yaml`, `mod.yaml`). The client currently imports these via Vite `?raw` at bundle time and merges them synchronously in `loadBaseContent()`. Rust tests embed a duplicate `pack.json` via `include_str!` in `packages/game-core/src/content/mod.rs`.

This works but creates drift risk: editing YAML does not automatically update `pack.json`, and bundled imports require a dev-server reload (not a full rebuild, but still compile-time coupling). The user wants a single YAML source loaded dynamically at application startup, with page refresh as the reload mechanism.

## Goals / Non-Goals

**Goals:**

- `content/base/*.yaml` is the **only** authoritative content source (remove `pack.json`)
- Client loads YAML via **runtime `fetch`** at app boot, before WASM `Game` is created
- Client and WASM receive the same merged `ContentPack` (JSON across the WASM boundary, unchanged)
- Rust unit/integration tests read the same YAML files from the repo filesystem
- No build scripts to generate intermediate JSON artifacts
- Loading screen covers the async boot window

**Non-Goals:**

- Hot-reloading content during an active simulation (page refresh is sufficient)
- User-facing mod picker or multi-mod merge
- Changing the YAML schema or simulation behavior
- Changing save file format or wire protocol

## Decisions

### 1. Runtime fetch instead of Vite `?raw` imports

**Decision:** Replace `import x from '@content/base/needs.yaml?raw'` with `fetch('/content/base/needs.yaml')` in an async `loadBaseContent()`.

**Rationale:** `?raw` bundles YAML into the JS chunk at build time. `fetch` reads the live files served as static assets — editing YAML + F5 picks up changes without touching the bundle.

**Alternative considered:** Keep `?raw` and only remove `pack.json` — rejected because it doesn't achieve runtime loading from a single live source.

### 2. Serve `content/` as static assets via Vite `publicDir`

**Decision:** Configure Vite `publicDir` to point at the repo-root `content/` directory (or symlink `content/base` into `packages/client/public/content/base`).

**Rationale:** Vite copies/serves `public/` at the URL root. `fetch('/base/needs.yaml')` or `fetch('/content/base/needs.yaml')` works in dev and production without custom middleware.

**Preferred approach:** Set `publicDir: resolve(__dirname, '../../content')` so URLs are `/base/needs.yaml`. Keep fetch paths aligned with the served structure.

**Alternative considered:** Custom Vite plugin to serve `../../content` — more complex, no benefit over `publicDir`.

### 3. Async boot orchestrated by `App.vue`

**Decision:** `App.vue` `onMounted` sequence:

```
1. await loadBaseContent()     // fetch + parse YAML
2. create PixiRenderer(content)
3. create GameManager()
4. gameManager.start(contentJson)
5. hide loading screen on worker 'ready'
```

**Rationale:** Single orchestration point; UI components receive content via props or `provide/inject` instead of module-level sync calls.

**Alternative considered:** Worker fetches YAML independently — rejected to avoid duplicate fetch/parse and keep one parse on main thread.

### 4. Pass content JSON to worker in `start` message

**Decision:** Extend `MainToWorkerMessage` `start` variant to include `contentJson: string`. Worker calls `Game::new(contentJson)` on receive.

**Rationale:** Main thread already parsed YAML; worker just needs the JSON string for WASM. Avoids second fetch in worker.

**Alternative considered:** Shared `SharedArrayBuffer` or importScripts — overkill for ~5 KB payload.

### 5. Remove `pack.json`; Rust tests read YAML from disk

**Decision:** Delete `content/base/pack.json`. Add a `#[cfg(test)]` helper in `game-core` that reads `content/base/*.yaml`, merges to JSON, and calls `ContentRegistry::from_json`. Use `serde_yaml` as a **dev-dependency** in a `test_support` module.

**Rationale:** Tests use the same files as the browser without build scripts. `cargo test` runs from repo root (or use `CARGO_MANIFEST_DIR`-relative paths up to `content/base/`).

**Alternative considered:** Keep `pack.json` only for tests — rejected; defeats single source of truth.

### 6. Inject content into UI components

**Decision:** Use Vue `provide/inject` with key `contentPack: ContentPack` set in `App.vue` after load. `Toolbar`, `ColonistInfo` inject it. `PixiRenderer` receives content in constructor.

**Rationale:** Minimal prop-drilling; components already mount after content is ready (hidden behind `loading` flag).

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Fetch 404 in dev/prod | Align `publicDir` and fetch URLs; add integration test or smoke check |
| UI renders before content ready | Keep `loading=true` until content + worker ready; don't mount toolbar until content injected |
| Rust test path resolution | Use `env!("CARGO_MANIFEST_DIR")` + `../../../content/base/` relative path; document that tests require repo layout |
| Extra network round-trips at boot | 5 parallel fetches (~few KB total); negligible vs WASM init |
| `PixiRenderer` module-level `loadBaseContent()` | Refactor to constructor param (called from App after async load) |

## Migration Plan

1. Configure Vite `publicDir` and verify `fetch('/base/needs.yaml')` returns 200 in dev
2. Implement async `loadBaseContent()` with parallel fetches
3. Refactor `App.vue` boot sequence and `provide/inject` for content
4. Extend worker `start` message with `contentJson`
5. Remove `?raw` imports and `vite-env.d.ts` YAML module declarations (if unused)
6. Add Rust test helper reading YAML; update all `base_content()` / `base_content_json()` call sites
7. Delete `pack.json`
8. Manual test: edit `buildings.yaml` label → F5 → toolbar shows new label

**Rollback:** Revert to `?raw` imports and restore `pack.json` from git history.

## Open Questions

- Exact `publicDir` URL prefix: `/base/...` vs `/content/base/...` — decide during implementation based on cleanest Vite config
- Whether to keep `mod.yaml` fetch (currently parsed but unused) — yes, for future mod metadata
