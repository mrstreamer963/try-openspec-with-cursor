## Context

Pause and speed are controlled via HUD buttons in `App.vue` → `Hud.vue`. Events flow `set_paused` / `set_speed` through `GameManager` to the WebWorker, which scales `BASE_DT * speed` before calling WASM `tick()`. HUD currently offers 1×, 2×, 3×. Game core clamps speed to `[0.1, 10.0]` — 5× and 10× fit without Rust changes.

## Goals / Non-Goals

**Goals:**
- Keyboard shortcuts: Space (pause toggle), 1 (1×), 2 (5×), 3 (10×)
- HUD buttons updated to 1×, 5×, 10×
- Shortcuts reuse existing `togglePause()` / `setSpeed()` — no new worker protocol

**Non-Goals:**
- Remapping keys or showing a key-bindings overlay
- Changing tick rate or game-core speed clamp beyond accepting 5 and 10
- Touch/mobile controls

## Decisions

### 1. Global `keydown` on `window` in `App.vue`

**Choice:** Register `window.addEventListener('keydown', …)` in `onMounted`, remove in `onUnmounted`.

**Rationale:** Matches RimWorld-style always-available controls. Canvas does not need focus. Reuses existing handler functions so HUD and keyboard stay in sync via snapshot feedback.

**Alternatives considered:**
- *Pixi canvas focus* — requires click-to-focus; worse UX
- *Separate composable* — overkill for four keys

### 2. Key mapping: digit keys select preset, not literal multiplier

**Choice:** `1` → 1×, `2` → 5×, `3` → 10× (HUD labels updated accordingly).

**Rationale:** User requested keys 1/2/3 with speeds x5 and x10. Keeping keys 2 and 3 as slot selectors avoids adding a fourth key for 10×.

### 3. Shared speed presets constant

**Choice:** Define `SPEED_PRESETS = [1, 5, 10] as const` in one place (e.g. `App.vue` or small `speedPresets.ts`) and pass to `Hud.vue` as prop.

**Rationale:** HUD buttons and keyboard handler must stay aligned; single source of truth prevents drift.

### 4. Prevent Space default

**Choice:** `event.preventDefault()` when handling Space to avoid page scroll.

**Rationale:** Space scrolls the page in browsers; game uses it for pause.

## Risks / Trade-offs

- **[Key 2/3 label mismatch]** Keys say "2" and "3" but set 5× and 10× → Mitigation: HUD shows actual multipliers (5×, 10×); optional tooltip later
- **[Space while typing]** No text inputs in v1 → No guard needed now; skip if `event.target` is input/textarea if added later
- **[10× simulation load]** More ticks-worth of work per interval → Acceptable at 20 Hz with tiny world; profile if needed
