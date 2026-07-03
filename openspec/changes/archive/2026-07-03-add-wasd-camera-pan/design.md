## Context

Camera controls live in `PixiRenderer.ts`. Panning is implemented via pointer drag (`pointerdown` / `pointermove`), zoom via mouse wheel. `CameraState` holds `offsetX`, `offsetY`, and `zoom`; `applyCamera()` applies them to `worldContainer`. The PixiJS ticker already runs each frame for rendering.

`GameSession.vue` registers a separate `window` `keydown` handler for Space (pause) and 1/2/3 (speed). WASD keys do not conflict with those shortcuts.

## Goals / Non-Goals

**Goals:**
- Pan the camera with W/A/S/D and arrow keys while keys are held
- Smooth, frame-rate-independent movement using delta time
- Diagonal pan when multiple direction keys are held
- Coexist with mouse drag pan and wheel zoom without regressions

**Non-Goals:**
- Edge-scroll panning (mouse at screen edge)
- Rebinding keys or a settings UI
- Camera bounds / clamping to world edges (not implemented for drag today)
- Touch on-screen D-pad for mobile

## Decisions

### 1. Implement keyboard pan in `PixiRenderer`, not `GameSession`

**Rationale:** All camera state and interaction (drag, wheel) already live in `PixiRenderer`. Keeping keyboard pan there avoids splitting camera logic across components.

**Alternative considered:** Add WASD handling in `GameSession.vue` and expose `panCamera(dx, dy)` on the renderer — rejected as unnecessary indirection.

### 2. Track pressed keys with `keydown` / `keyup` and apply pan on the Pixi ticker

**Rationale:** Holding a key requires continuous movement. A per-frame ticker callback reads the key set, computes a direction vector, and updates `camera.offsetX/Y` by `speed * deltaSeconds`. This matches RTS conventions and supports diagonal movement when W+D are both held.

**Alternative considered:** Repeat `keydown` events only — jerky and OS-dependent repeat rate.

### 3. Pan speed: fixed screen pixels per second (not zoom-scaled)

**Rationale:** At higher zoom the world moves faster in tile units if speed is constant in screen space, which feels natural (you see more detail, so the map scrolls at a comfortable screen rate). Drag pan is inherently screen-pixel based; matching that keeps keyboard and mouse pan consistent.

**Constant:** `KEYBOARD_PAN_SPEED = 600` px/s (tunable during implementation).

### 4. Supported keys

| Direction | Keys |
|-----------|------|
| Up | `KeyW`, `ArrowUp` |
| Down | `KeyS`, `ArrowDown` |
| Left | `KeyA`, `ArrowLeft` |
| Right | `KeyD`, `ArrowRight` |

Use `event.code` (not `event.key`) so layout-independent and to avoid conflict with digit speed keys.

### 5. `preventDefault` on arrow keys only

Arrow keys can scroll the page in the browser. Call `preventDefault()` for arrow keys when the game canvas is active. WASD keys do not need it.

### 6. Ignore keys when typing in an input

If `event.target` is an `input`, `textarea`, or `select`, skip pan key handling (future-proof; no text inputs in game view today).

## Risks / Trade-offs

- **[Risk] Arrow keys scroll the browser** → `preventDefault` on arrow keydown while renderer is active
- **[Risk] Key stuck after alt-tab** → Clear key set on `window` `blur` event
- **[Risk] Listener leak on destroy** → Remove keydown/keyup/blur listeners in `destroy()` alongside existing pointer listeners
- **[Trade-off] No camera bounds** → Same as drag pan today; edges of world can scroll off-screen
