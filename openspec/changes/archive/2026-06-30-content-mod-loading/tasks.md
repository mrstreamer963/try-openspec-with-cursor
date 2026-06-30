## 1. Content layout

- [x] 1.1 Add `content/mods.yaml` with `mods: [base]`
- [x] 1.2 Add demo `content/mods/hardmode/` with `mod.yaml` and `needs.yaml` override

## 2. Client merge and load

- [x] 2.1 Add `mergeContent.ts` with `mergeById` and `mergeContentPacks`
- [x] 2.2 Add `loadContent.ts` (manifest, per-mod fetch, merge)
- [x] 2.3 Refactor `loadBaseContent.ts` to delegate to `loadContent()`
- [x] 2.4 Wire `App.vue` to `loadContent()` and track `modIds`

## 3. Save/load metadata

- [x] 3.1 Add optional `content_mods` to `SaveFile` and `buildSaveFile`
- [x] 3.2 Update `validateSaveFile` to return `content_mods`
- [x] 3.3 Mod mismatch confirm dialog in `App.vue` on load

## 4. Tests

- [x] 4.1 Add Vitest and unit tests for merge logic
