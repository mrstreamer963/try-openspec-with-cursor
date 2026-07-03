## 1. Keyboard pan state in PixiRenderer

- [x] 1.1 Add a `Set<string>` (or equivalent) to track currently pressed pan key codes (`KeyW`, `KeyA`, `KeyS`, `KeyD`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`)
- [x] 1.2 Register `keydown` / `keyup` listeners on `window` in `setupInteraction()`; add keys to the set on down, remove on up; skip when `event.target` is an input/textarea/select
- [x] 1.3 Call `preventDefault()` on arrow-key `keydown` to avoid browser scroll
- [x] 1.4 Clear the pressed-key set on `window` `blur` to avoid stuck keys after alt-tab

## 2. Per-frame camera movement

- [x] 2.1 Extend the existing Pixi ticker callback (or add a dedicated one) to read pressed keys each frame, compute a normalized direction vector, and update `camera.offsetX` / `camera.offsetY` by `KEYBOARD_PAN_SPEED * deltaSeconds`
- [x] 2.2 Call `applyCamera()` after keyboard offset updates (reuse existing ticker if practical)

## 3. Cleanup and verification

- [x] 3.1 Remove `keydown`, `keyup`, and `blur` listeners in `destroy()` alongside existing pointer/wheel cleanup
- [x] 3.2 Manually verify: W/A/S/D and arrows pan smoothly; diagonal works; mouse drag and wheel zoom still work; Space/1/2/3 shortcuts unaffected
