## 1. Static asset serving

- [x] 1.1 Configure Vite `publicDir` to serve repo-root `content/` directory
- [x] 1.2 Verify `fetch('/base/needs.yaml')` returns 200 in dev server

## 2. Async content loader (client)

- [x] 2.1 Rewrite `loadBaseContent.ts` as async function using parallel `fetch` for all YAML files
- [x] 2.2 Remove `?raw` YAML imports and unused `vite-env.d.ts` module declarations
- [x] 2.3 Add error handling with descriptive messages for fetch/parse failures
- [x] 2.4 Keep `contentPackToJson`, color maps, and label helpers working with async-loaded pack

## 3. Boot sequence and content injection (client)

- [x] 3.1 Refactor `App.vue` to `await loadBaseContent()` before starting worker and renderer
- [x] 3.2 Add `provide/inject` for `ContentPack` to child components
- [x] 3.3 Update `Toolbar.vue` and `ColonistInfo.vue` to inject content instead of sync module-level load
- [x] 3.4 Update `PixiRenderer.ts` to accept `ContentPack` in constructor (remove module-level load)
- [x] 3.5 Keep loading screen visible until content load completes; show error on failure

## 4. Worker bridge

- [x] 4.1 Extend `MainToWorkerMessage` `start` variant with `contentJson: string`
- [x] 4.2 Update `gameWorker.ts` to use `contentJson` from start message instead of calling `loadBaseContent()`
- [x] 4.3 Update `GameManager.start()` to accept and forward `contentJson`

## 5. Rust test helper and cleanup

- [x] 5.1 Add `serde_yaml` dev-dependency to `game-core/Cargo.toml`
- [x] 5.2 Create test helper that reads `content/base/*.yaml` from filesystem and merges to JSON
- [x] 5.3 Replace `base_content_json()` / `include_str!(pack.json)` with test helper in all test call sites
- [x] 5.4 Delete `content/base/pack.json`

## 6. Verification

- [x] 6.1 Run `cargo test` — all content and systems tests pass
- [x] 6.2 Run `npm run dev` — game loads with toolbar labels and colors from YAML
- [x] 6.3 Edit a building label in `buildings.yaml`, refresh page — toolbar shows updated label without WASM rebuild
