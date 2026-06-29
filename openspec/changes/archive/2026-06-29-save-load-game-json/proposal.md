## Why

Players cannot preserve colony progress across browser sessions. Without save/load, any refresh or accidental tab close resets the world. The game core already emits a serializable `StateSnapshot` on every tick — exposing it as a downloadable JSON file and restoring simulation from that file is a natural next step for a playable colony sim.

## What Changes

- Define a **save file format** (JSON) based on `StateSnapshot` with a version field for forward compatibility
- Add **Save** in the HUD: serialize the current snapshot and trigger a browser file download (`.json`)
- Add **Load** in the HUD: pick a JSON file, validate it, and restore the full ECS world in the game core
- Add `IncomingEvent::LoadState` (or equivalent) so the WASM core rebuilds world grid, buildings, colonists, construction sites, pause, and speed from the snapshot
- Pause simulation during load; resume only after a successful restore and snapshot broadcast
- Show clear user feedback on invalid or incompatible save files (error toast or inline message)

## Capabilities

### New Capabilities

- `game-state-persistence`: Save file schema, validation rules, save-to-file and load-from-file user flows, and error handling for corrupt or incompatible saves

### Modified Capabilities

- `game-core`: Add state restoration from a validated snapshot; expose save payload generation aligned with the persistence schema
- `worker-bridge`: Route save/load commands between main thread and WASM; support one-shot load that replaces the running simulation
- `view-layer`: HUD controls for Save and Load; file picker and download UX

## Impact

- `packages/game-core/src/events.rs` — optional save schema wrapper; new `LoadState` incoming event
- `packages/game-core/src/game.rs` — `restore_from_snapshot` rebuilding ECS + grid
- `packages/client/src/worker/gameWorker.ts` — handle load event and snapshot export for save
- `packages/client/src/game/protocol.ts` — typed messages for save/load if needed on main thread
- `packages/client/src/components/Hud.vue` — Save / Load buttons
- `packages/client/src/App.vue` — wire file download and file input handlers
- `openspec/specs/game-core/spec.md`, `worker-bridge/spec.md`, `view-layer/spec.md` — delta updates via this change
