## 1. Tile geometry helpers

- [x] 1.1 Add `tileShapes.ts` with `horizontalVerticalLineTiles` and `rectTiles`
- [x] 1.2 Add unit tests for line and rectangle tile enumeration

## 2. PixiRenderer interaction

- [x] 2.1 Add `setToolMode`, preview layer, and interaction intent (RMB pan, LMB shapes)
- [x] 2.2 Implement wall line and deconstruct rectangle preview drawing
- [x] 2.3 Add `onSceneLineBuild` and `onSceneRectDeconstruct` callbacks; suppress colonist pick in tool modes
- [x] 2.4 Suppress context menu on canvas; remove LMB drag pan

## 3. GameSession wiring

- [x] 3.1 Pass `toolMode` to renderer on toolbar change
- [x] 3.2 Filter valid tiles and send batch build/deconstruct events on drag commit

## 4. Verification

- [x] 4.1 Run client unit tests
