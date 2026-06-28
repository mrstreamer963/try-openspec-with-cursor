## 1. Speed presets

- [x] 1.1 Add shared `SPEED_PRESETS = [1, 5, 10]` constant for HUD and keyboard
- [x] 1.2 Update `Hud.vue` to render 1×, 5×, 10× from presets prop

## 2. Keyboard shortcuts

- [x] 2.1 Add `window` keydown listener in `App.vue` (mount/unmount lifecycle)
- [x] 2.2 Map Space → `togglePause()`, 1 → 1×, 2 → 5×, 3 → 10× via `setSpeed()`
- [x] 2.3 Call `preventDefault()` on Space to avoid page scroll

## 3. Verification

- [x] 3.1 Manually verify HUD buttons and keyboard shortcuts both set 1×/5×/10× and pause toggle
- [x] 3.2 Confirm worker passes scaled dt (0.25 at 5×, 0.50 at 10×) during fast-forward
