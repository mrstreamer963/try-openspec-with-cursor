## Context

`PixiRenderer.setupInteraction` treats any pointer drag as camera pan and only fires a tile click when movement stays within 4px. `GameSession` sends one `build` or `deconstruct` event per click. The renderer has no knowledge of `toolMode`, so it cannot branch drag behavior by active tool.

Keyboard pan (WASD/arrows) and wheel zoom already exist. The spec currently says "mouse drag" pans without specifying which button.

## Goals / Non-Goals

**Goals:**

- RMB drag pans the camera in all tool modes
- Wall tool: LMB drag commits an H/V line of build orders; preview during drag
- Deconstruct tool: LMB drag commits build orders for all deconstructible tiles in an axis-aligned rectangle; preview during drag
- Skip invalid tiles client-side before sending events (empty terrain, occupied tiles, etc.)
- Single-tap LMB still works for one tile in wall and deconstruct modes
- In active build/deconstruct modes, LMB does not open the colonist info panel

**Non-Goals:**

- Diagonal wall lines
- Batch worker events (`build_line` / `deconstruct_rect`) — send sequential existing events
- Deconstruct line mode (rectangle only)
- LMB drag pan in select mode (RMB replaces it)

## Decisions

### 1. Interaction intent from `(button, toolMode)`

| Input | Select | Wall | Deconstruct | Bed/Bush |
|-------|--------|------|-------------|----------|
| LMB tap | colonist / dismiss | 1 wall | 1 deconstruct | 1 build |
| LMB drag | ignored | H/V line | rectangle | ignored |
| RMB drag | pan | pan | pan | pan |

**Rationale:** Clear separation — LMB acts on the map, RMB moves the camera. Matches RTS/editor conventions.

**Alternatives considered:** MMB pan (rejected — user chose RMB); LMB pan in select only (rejected — inconsistent once wall drag exists).

### 2. H/V line via dominant axis

Compare `|dx|` and `|dy|` in tile space from drag start to current tile. Larger axis wins; tie goes horizontal. Range is inclusive on the fixed axis.

### 3. Client-side tile enumeration in `tileShapes.ts`

Pure functions `horizontalVerticalLineTiles` and `rectTiles`, unit-tested. `PixiRenderer` converts pointer coords to tiles and calls helpers.

### 4. Preview layer

New `previewLayer` container between `buildingsLayer` and `entitiesLayer`. Cleared on pointer up or tool mode change. Wall preview uses semi-transparent building color; deconstruct preview uses semi-transparent red fill.

### 5. Validity filtering in `GameSession`

On batch commit, filter tiles using the latest snapshot:

- **Build:** walkable terrain, no building, no construction site at tile
- **Deconstruct:** finished building, construction site, or existing deconstruction site at tile

Invalid tiles are omitted; no error toast per skipped tile.

### 6. `setToolMode` on renderer

`GameSession` calls `renderer.setToolMode(toolMode)` when toolbar selection changes. Renderer stores mode and clears preview on change.

## Risks / Trade-offs

- **[Risk] Trackpads without RMB** → Mitigation: WASD/arrow pan remains; document RMB requirement
- **[Risk] Browser context menu on RMB** → Mitigation: `contextmenu` preventDefault on canvas
- **[Risk] Many sequential worker messages on large rectangles** → Mitigation: acceptable for 50×50 max; batch event is future optimization
