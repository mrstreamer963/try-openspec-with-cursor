## Context

The WASM game core already serializes full world state as `StateSnapshot` (tiles, buildings, construction sites, colonists, pause, speed) on every tick via `OutgoingEvent::StateSnapshot`. The main thread keeps the latest snapshot in `GameManager.latestSnapshot`. There is no reverse path: `Game::new()` always generates a fresh world from a fixed seed and spawns three colonists.

Save/load is therefore a cross-cutting feature: a versioned JSON envelope on the client, a new `IncomingEvent::LoadState` in the event protocol, ECS/grid rebuild logic in Rust, and HUD controls for download / file picker.

## Goals / Non-Goals

**Goals:**
- One-click **Save** downloads a `.json` file representing the current colony
- **Load** replaces the running simulation with validated file contents
- Save file includes `version` for future schema changes
- Invalid files show a clear error without corrupting the running game
- After successful load, HUD, renderer, and worker reflect restored pause/speed

**Non-Goals:**
- Autosave, multiple save slots, or cloud sync
- Saving in-progress colonist paths, bed reservations, or `SleepingOnBed` timers (restored colonists get `Task { kind }` from snapshot and idle path state; auto-assign re-resolves targets on next tick)
- Server-side persistence or localStorage (file-based only)
- Editing terrain via save files (tiles are restored as-is for integrity; procedural re-generation from seed is not used on load)

## Decisions

### 1. Save file schema: versioned wrapper around `StateSnapshot`

**Choice:** Define `SaveFile { version: u32, saved_at: String, state: StateSnapshot }` with `version = 1`.

**Rationale:** `StateSnapshot` is already serde-stable and shared between Rust and TypeScript. A thin wrapper enables future fields (metadata, checksum) without breaking raw snapshot parsing in tests.

**Alternatives considered:**
- *Raw `StateSnapshot` only* — simpler but no migration hook
- *Separate save format duplicating fields* — drift risk between snapshot and save schema

### 2. Save on main thread from latest snapshot

**Choice:** `App.vue` builds the save JSON from `GameManager.snapshot` (no worker round-trip).

**Rationale:** Main thread already receives every snapshot; avoids new worker message type for export. Save is instantaneous and matches what the player sees.

**Alternatives considered:**
- *Worker `get_snapshot`* — redundant; snapshot already mirrored on main thread

### 3. Load via `IncomingEvent::LoadState { state: StateSnapshot }`

**Choice:** Main thread parses and validates the file, then sends `load_state` through existing `GameManager.sendEvent` → worker → WASM `handle_event`.

**Rationale:** Reuses the event pipeline; game core owns authoritative restore logic.

### 4. Restore by clearing ECS and rebuilding grid + entities

**Choice:** `Game::restore_from_snapshot(snapshot)`:
1. Despawn all colonist, building, and construction-site entities
2. Rebuild `WorldGrid.terrain` from `tiles`; clear and repopulate `buildings` from snapshot buildings
3. Spawn building entities (with `BerrySupply` when `berries` present)
4. Spawn construction sites with `work_remaining` derived from `progress`
5. Spawn colonists with ids, names, positions, needs, buff markers, and `Task { kind, ..default }`
6. Set `paused` and `speed`; insert updated `WorldGrid` resource

**Rationale:** Snapshot is the source of truth for visible/simulated state. Transient ECS components (paths, reservations) are safely reset.

**Alternatives considered:**
- *WASM `Game` reinstantiation* — would reset worker-local `paused`/`speed` vars awkwardly; in-place restore is cleaner

### 5. Client-side validation before dispatch

**Choice:** TypeScript validates: `version === 1`, `state.tiles.length === 2500`, required arrays present, numeric ranges sane. On failure, show error and do not send `LoadState`.

**Rationale:** Fail fast with user-friendly messages; WASM still validates and returns `OutgoingEvent::Error` as a second line of defense.

### 6. File download via Blob + temporary `<a download>`

**Choice:** `saveGame.ts` helper: `JSON.stringify(saveFile, null, 2)`, `Blob`, object URL, programmatic click. Filename: `colony-save-{ISO-date}.json`.

**Rationale:** Standard browser pattern; no dependencies.

### 7. Hidden `<input type="file" accept=".json">` for load

**Choice:** HUD emits `load`; `App.vue` triggers hidden file input, reads `File.text()`, parses, validates, dispatches load event.

**Rationale:** Native file picker; works in dev and production without File System Access API complexity.

## Risks / Trade-offs

- **[Incomplete transient state]** Paths and in-progress sleep timers are not saved → Mitigation: colonists resume from task kind; movement re-paths on next tick; document in save UX tooltip if needed
- **[Corrupt JSON]** Parse errors → Mitigation: client validation + WASM error event; running game unchanged on failure
- **[Version mismatch]** Future schema changes → Mitigation: reject unknown `version` with message to update game
- **[Load during fast-forward]** Race with tick loop → Mitigation: load is synchronous in worker handler before next tick; snapshot posted immediately after restore

## Migration Plan

Not applicable — additive feature. No changes to existing event types. Old saves (none exist) N/A.

## Open Questions

_(none — scope is sufficient for v1)_
