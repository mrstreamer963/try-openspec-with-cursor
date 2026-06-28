## Why

Players currently control pause and simulation speed only via HUD buttons. Keyboard shortcuts (Space, 1, 2, 3) are standard in colony sims and make speed control faster during play. The existing 2× and 3× speeds are too slow for idle-style fast-forward; 5× and 10× better match the intended pacing.

## What Changes

- Add keyboard shortcuts: **Space** toggles pause/resume, **1** sets 1×, **2** sets 5×, **3** sets 10×
- Replace HUD speed buttons from 1×/2×/3× to 1×/5×/10×
- Update worker speed multiplier to support 1×, 5×, and 10× (replacing 2× and 3×)
- Ignore shortcuts when focus is in a text input (if any); canvas/game view receives keys globally on the page

## Capabilities

### New Capabilities

_(none — behavior extends existing view-layer and worker-bridge requirements)_

### Modified Capabilities

- `view-layer`: HUD speed presets change to 1×/5×/10×; add keyboard shortcut requirement for pause and speed
- `worker-bridge`: Speed multiplier requirement updated from 1×/2×/3× to 1×/5×/10×

## Impact

- `packages/client/src/App.vue` — global keydown listener for Space/1/2/3
- `packages/client/src/components/Hud.vue` — speed button labels and values (1, 5, 10)
- `packages/client/src/worker/gameWorker.ts` — no logic change beyond accepting new multiplier values
- `openspec/specs/view-layer/spec.md` and `openspec/specs/worker-bridge/spec.md` — requirement updates via delta specs
