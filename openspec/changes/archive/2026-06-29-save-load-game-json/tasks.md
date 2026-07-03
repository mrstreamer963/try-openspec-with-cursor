## 1. Game core — load state event and restore

- [x] 1.1 Add `IncomingEvent::LoadState { state: StateSnapshot }` to `events.rs` (serde snake_case `load_state`)
- [x] 1.2 Implement `Game::restore_from_snapshot`: despawn entities, rebuild `WorldGrid` from tiles/buildings, respawn buildings (with `BerrySupply`), construction sites (derive `work_remaining` from progress), and colonists (needs, buff markers, `Task.kind`)
- [x] 1.3 Wire `LoadState` in `dispatch`; validate tile count (2500) and return `OutgoingEvent::Error` on failure without mutating state
- [x] 1.4 Add Rust unit tests: round-trip save snapshot → restore → `get_snapshot` matches key fields; invalid tile count rejected

## 2. Client types and persistence helpers

- [x] 2.1 Add `SaveFile` type and `SAVE_VERSION = 1` in `packages/client/src/game/saveFile.ts`
- [x] 2.2 Implement `buildSaveFile(snapshot)` and `validateSaveFile(raw): StateSnapshot | error message`
- [x] 2.3 Implement `downloadSaveFile(saveFile)` using Blob + temporary anchor download
- [x] 2.4 Extend `IncomingEvent` in `types.ts` with `load_state`; update `serializeIncomingEvent` / protocol if needed

## 3. Worker bridge

- [x] 3.1 Handle `load_state` in `gameWorker.ts` `handleEvent`: dispatch to WASM, sync local `paused` and `speed` from loaded snapshot, post snapshot or error

## 4. View layer — HUD and wiring

- [x] 4.1 Add Save and Load buttons to `Hud.vue`; emit `save` and `load` events
- [x] 4.2 In `App.vue`: Save handler builds save file from `GameManager.snapshot` and triggers download; show error if no snapshot
- [x] 4.3 In `App.vue`: hidden file input for Load; parse, validate, `sendEvent({ type: 'load_state', state })`; surface validation/worker errors to user
- [x] 4.4 Ensure renderer and HUD reflect loaded pause/speed from incoming snapshot

## 5. Verification

- [x] 5.1 Manual test: play, build, pause, save — reopen file and confirm JSON structure (`version`, `saved_at`, `state`)
- [x] 5.2 Manual test: load saved file — world, colonists, buildings, construction sites, pause, and speed match saved state
- [x] 5.3 Manual test: invalid JSON and wrong `version` show error without breaking current game
- [x] 5.4 Run `cargo test` in `packages/game-core` and client typecheck/build
